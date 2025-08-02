use std::vec;

use entity::course_section_item::ContentType;
use sea_orm::{sea_query, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_plugin_http::reqwest;
use tauri_plugin_store::StoreExt;

use crate::{
	auth::auth_keys,
	database::DatabaseState,
	request::rest::{
		self, build_rest_request, GetCourseSectionsFunctionData, GetCoursesFunctionData, RestResponse,
	},
	sync::{self},
};

#[derive(Serialize, Deserialize, Type)]
pub struct CourseWithSections {
	pub course: entity::course::Model,
	pub sections: Vec<CourseSectionWithItems>,
}

#[derive(Serialize, Deserialize, Type)]
pub struct CourseSectionWithItems {
	pub section: entity::course_section::Model,
	pub items: Vec<entity::course_section_item::Model>,
}

#[tauri::command]
#[specta::specta]
pub async fn get_course(
	app: AppHandle,
	state: tauri::State<'_, DatabaseState>,
	course_id: i32,
) -> Result<CourseWithSections, String> {
	sync::revalidate_task(
		&app,
		format!("get_course_{}", course_id).as_str(),
		"Get course state",
		async move |app_handle| {
			let store = app_handle.store("store.json").unwrap();
			let client = reqwest::Client::new();
			let rest_functions = vec![rest::get_course_sections(course_id, true)];

			let host = store.get(auth_keys::MOODLE_HOST).unwrap();
			let ws_token = store.get(auth_keys::WS_TOKEN).unwrap();
			let request = build_rest_request(
				&client,
				&host.as_str().unwrap(),
				ws_token.as_str().unwrap(),
				rest_functions,
			)
			.map_err(|e| e.to_string())?;

			let response = client.execute(request).await.unwrap();
			if response.status().is_success() == false {
				return Err(format!("Failed to fetch course sections with id: {course_id}").into());
			}

			let body = response.text().await.unwrap();
			if body.contains("errorcode") {
				return Err(format!("Could not get user courses: {}", body).into());
			}

			let rest_response: RestResponse = serde_json::from_str(&body).map_err(|e| e.to_string())?;
			let data_str = rest_response
				.responses
				.get(0)
				.and_then(|r| r.data.as_ref())
				.ok_or_else(|| "No data found in response".to_string())?;
			let parsed_body = serde_json::from_value::<GetCourseSectionsFunctionData>(
				serde_json::from_str(data_str).map_err(|e| e.to_string())?,
			)?;
			let sections_data = match parsed_body {
				GetCourseSectionsFunctionData::Sections(sections) => sections,
			};

			let sections = sections_data
				.iter()
				.map(|section| entity::course_section::ActiveModel {
					id: ActiveValue::Set(section.id),
					name: ActiveValue::Set(section.name.clone()),
					course_id: ActiveValue::Set(course_id),
				})
				.collect::<Vec<_>>();

			let items = sections_data
				.iter()
				.flat_map(|section| {
					section
						.modules
						.iter()
						.map(|module| entity::course_section_item::ActiveModel {
							id: ActiveValue::Set(module.id),
							name: ActiveValue::Set(module.name.clone()),
							section_id: ActiveValue::Set(section.id),
							content_type: ActiveValue::Set(ContentType::Page), // todo: map to actual type
							updated_at: ActiveValue::NotSet,
						})
				})
				.collect::<Vec<_>>();

			let state = app_handle.state::<DatabaseState>();
			let db = &state.0;
			let txn = db.begin().await.map_err(|e| e.to_string())?;
			for section in sections {
				entity::CourseSection::insert(section)
					.on_conflict(
						sea_query::OnConflict::column(entity::course_section::Column::Id)
							.update_columns([
								entity::course_section::Column::Name,
								entity::course_section::Column::CourseId,
							])
							.to_owned(),
					)
					.exec(&txn)
					.await
					.map_err(|e| e.to_string())?;
			}

			for item in items {
				entity::CourseSectionItem::insert(item)
					.on_conflict(
						sea_query::OnConflict::column(entity::course_section_item::Column::Id)
							.update_columns([
								entity::course_section_item::Column::Name,
								entity::course_section_item::Column::SectionId,
								entity::course_section_item::Column::ContentType,
							])
							.to_owned(),
					)
					.exec(&txn)
					.await
					.map_err(|e| e.to_string())?;
			}

			txn.commit().await.map_err(|e| e.to_string())?;
			Ok(())
		},
	)
	.await;

	let db = &state.0;
	let (course, sections) = entity::Course::find_by_id(course_id)
		.find_with_related(entity::CourseSection)
		.all(db)
		.await
		.map_err(|e| e.to_string())?
		.into_iter()
		.next()
		.ok_or_else(|| format!("Course with id {} not found", course_id))?;

	let mut sections_with_items = vec![];
	for section in sections {
		let items = entity::CourseSectionItem::find()
			.filter(entity::course_section_item::Column::SectionId.eq(section.id))
			.all(db)
			.await
			.map_err(|e| e.to_string())?;
		sections_with_items.push((section, items));
	}

	Ok(CourseWithSections {
		course: course,
		sections: sections_with_items
			.into_iter()
			.map(|(section, items)| CourseSectionWithItems { section, items })
			.collect(),
	})
}

// #[tauri::command]
// #[specta::specta]
// pub async fn get_module_content(
// 	app: AppHandle,
// 	state: tauri::State<'_, DatabaseState>,
// 	module_id: i32,
// ) -> Result<entity::module_content::Model, String> {
// 	sync::revalidate_task(
// 		&app,
// 		format!("get_module_{}", module_id).as_str(),
// 		"Get module",
// 		async move |app_handle| {},
// 	)
// 	.await;

// 	let db = &state.0;
// 	entity::ModuleContent::find_by_id(module_id)
// 		.one(db)
// 		.await
// 		.map_err(|error| error.to_string())?
// 		.ok_or_else(|| format!("Module with id {} not found", module_id))
// }

#[tauri::command]
#[specta::specta]
pub async fn get_user_courses(
	app: AppHandle,
	state: tauri::State<'_, DatabaseState>,
) -> Result<Vec<entity::course::Model>, String> {
	sync::revalidate_task(
		&app,
		"get_user_courses",
		"Get user courses",
		async move |app_handle| {
			let store = app_handle.store("store.json").unwrap();
			let client = reqwest::Client::new();
			let user_id = store
				.get(auth_keys::USER_ID)
				.and_then(|id| id.as_str().and_then(|s| s.parse::<u32>().ok()))
				.ok_or("Failed to retrieve user id from store")?;
			let rest_functions = vec![rest::get_user_courses(user_id)];

			let host = store.get(auth_keys::MOODLE_HOST).unwrap();
			let ws_token = store.get(auth_keys::WS_TOKEN).unwrap();
			let request = build_rest_request(
				&client,
				&host.as_str().unwrap(),
				ws_token.as_str().unwrap(),
				rest_functions,
			)
			.map_err(|e| e.to_string())?;

			let response = client.execute(request).await.unwrap();
			if response.status().is_success() == false {
				return Err(
					format!(
						"Could not get user courses: {}",
						response.text().await.unwrap()
					)
					.into(),
				);
			}

			let body = response.text().await.unwrap();
			if body.contains("errorcode") {
				return Err(format!("Could not get user courses: {}", body).into());
			}

			// a little annoying, but data returned from the rest api is usually stringified json
			// here we parse the body into RestResponse, which contains a vector of "responses" (discrete outputs of rest functions)
			// we then extract the first response, which contains the data we want
			// finally, we parse the data string into GetCoursesFunctionData, which contains the courses
			// todo: maybe just move this to a function
			let rest_response: RestResponse = serde_json::from_str(&body).map_err(|e| e.to_string())?;
			let data_str = rest_response
				.responses
				.get(0)
				.and_then(|r| r.data.as_ref())
				.ok_or_else(|| "No data found in response".to_string())?;
			let parsed_body = serde_json::from_value::<GetCoursesFunctionData>(
				serde_json::from_str(data_str).map_err(|e| e.to_string())?,
			)?;
			let courses = match parsed_body {
				GetCoursesFunctionData::Courses(courses) => courses
					.into_iter()
					.map(|course| entity::course::ActiveModel {
						id: ActiveValue::Set(course.id),
						name: ActiveValue::Set(course.full_name.clone()),
						colour: ActiveValue::Set(Some("brown".to_string())),
						icon: ActiveValue::NotSet,
					})
					.collect::<Vec<_>>(),
			};

			let state = app_handle.state::<DatabaseState>();
			let db = &state.0;
			let txn = db.begin().await.map_err(|e| e.to_string())?;
			for course in courses {
				entity::Course::insert(course)
					.on_conflict(
						sea_query::OnConflict::column(entity::course::Column::Id)
							.update_columns([
								entity::course::Column::Name,
								entity::course::Column::Colour,
								entity::course::Column::Icon,
							])
							.to_owned(),
					)
					.exec(&txn)
					.await
					.map_err(|e| e.to_string())?;
			}

			txn.commit().await.map_err(|e| e.to_string())?;
			Ok(())
		},
	)
	.await;

	let db = &state.0;
	entity::Course::find()
		.all(db)
		.await
		.map_err(|e| e.to_string())
		.map(|courses| courses.into_iter().collect())
}
