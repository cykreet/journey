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
	request::service_request::{
		build_service_request, service_methods, ServiceMethod, ServiceResponse,
	},
	store_keys,
	sync::{self},
};

#[derive(Default, Deserialize)]
struct ServiceCourses {
	courses: Vec<ServiceCourse>,
}

#[derive(Deserialize)]
struct ServiceCourse {
	id: i32,
	#[serde(rename = "fullname")]
	full_name: String,
}

#[derive(Default, Deserialize)]
struct ServiceCourseState {
	section: Vec<ServiceCourseStateSection>,
	#[serde(rename = "cm")]
	module: Vec<ServiceCourseStateModule>,
}

#[derive(Default, Deserialize)]
struct ServiceCourseStateSection {
	id: String,
	title: String,
}

#[derive(Default, Deserialize)]
struct ServiceCourseStateModule {
	id: String,
	name: String,
	#[serde(rename = "sectionid")]
	section_id: String,
	// describes the type of the module, i.e. forum, page, etc
	#[serde(rename = "module")]
	module_type: String,
}

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
			let auth_store = app_handle.store(store_keys::AUTH).unwrap();
			let client = reqwest::Client::new();
			let service_method =
				ServiceMethod::new(0, service_methods::GET_COURSE_STATE).with_course_id(course_id);

			let host = auth_store.get(auth_keys::MOODLE_HOST).unwrap();
			let session_cookie = auth_store.get(auth_keys::MOODLE_SESSION).unwrap();
			let session_key = auth_store.get(auth_keys::SESSION_KEY).unwrap();
			let request = build_service_request(
				&client,
				&host.as_str().unwrap(),
				session_cookie.as_str().unwrap(),
				session_key.as_str().unwrap(),
				vec![service_method],
			)
			.unwrap();

			let response = client.execute(request).await.unwrap();
			if response.status().is_success() == false {
				let message = format!(
					"Failed to fetch course sections for course with id: {course_id}",
					course_id = course_id
				);

				return Err(message.into());
			}

			let body = response.text().await.unwrap();
			if body.contains("errorcode") {
				let message = format!("Could not get user courses: {}", body);
				return Err(message.into());
			}

			let service_body = serde_json::from_str::<Vec<ServiceResponse<String>>>(&body).unwrap();
			let service_parsed = service_body
				.into_iter()
				.next()
				.unwrap()
				.data
				.unwrap_or_default();
			let course_state = serde_json::from_str::<ServiceCourseState>(&service_parsed).unwrap();
			let sections =
				course_state
					.section
					.iter()
					.map(|section| entity::course_section::ActiveModel {
						id: ActiveValue::Set(section.id.parse().unwrap()),
						name: ActiveValue::Set(section.title.clone()),
						course_id: ActiveValue::Set(course_id),
					});

			let items = course_state
				.module
				.iter()
				.map(|module| entity::course_section_item::ActiveModel {
					id: ActiveValue::Set(module.id.parse().unwrap()),
					name: ActiveValue::Set(module.name.clone()),
					section_id: ActiveValue::Set(module.section_id.parse().unwrap()),
					// todo: map module_type to ContentType enum
					content_type: ActiveValue::Set(ContentType::Page),
					updated_at: ActiveValue::NotSet,
				})
				.collect::<Vec<_>>();

			let state = app_handle.state::<DatabaseState>();
			let db = &state.0;
			let txn = db.begin().await.map_err(|error| error.to_string())?;
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
					.map_err(|error| error.to_string())?;
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
					.map_err(|error| error.to_string())?;
			}

			txn.commit().await.map_err(|error| error.to_string())?;
			Ok(())
		},
	)
	.await;

	let db = &state.0;
	let course_with_sections = entity::Course::find_by_id(course_id)
		.find_with_related(entity::CourseSection)
		.all(db)
		.await
		.map_err(|error| error.to_string())?
		.into_iter()
		.next()
		.ok_or_else(|| format!("Course with id {} not found", course_id))?;

	let mut sections_with_items = vec![];
	for section in course_with_sections.1 {
		let items = entity::CourseSectionItem::find()
			.filter(entity::course_section_item::Column::SectionId.eq(section.id))
			.all(db)
			.await
			.map_err(|error| error.to_string())?;
		sections_with_items.push((section, items));
	}

	Ok(CourseWithSections {
		course: course_with_sections.0,
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
			let auth_store = app_handle.store(store_keys::AUTH).unwrap();
			let client = reqwest::Client::new();
			let service_method = ServiceMethod::new(0, service_methods::GET_COURSES)
				.with_offset(0)
				.with_limit(0)
				.with_classification("all")
				.with_sort("fullname");

			let host = auth_store.get(auth_keys::MOODLE_HOST).unwrap();
			let session_cookie = auth_store.get(auth_keys::MOODLE_SESSION).unwrap();
			let session_key = auth_store.get(auth_keys::SESSION_KEY).unwrap();
			let request = build_service_request(
				&client,
				&host.as_str().unwrap(),
				session_cookie.as_str().unwrap(),
				session_key.as_str().unwrap(),
				vec![service_method],
			)
			.unwrap();

			let response = client.execute(request).await.unwrap();
			if response.status().is_success() == false {
				let message = format!(
					"Could not get user courses: {}",
					response.text().await.unwrap()
				);

				return Err(message.into());
			}

			let body = response.text().await.unwrap();
			if body.contains("errorcode") {
				let message = format!("Could not get user courses: {}", body);
				return Err(message.into());
			}

			let service_body: Vec<ServiceResponse<ServiceCourses>> = serde_json::from_str(&body).unwrap();
			let service_parsed = service_body
				.into_iter()
				.next()
				.unwrap()
				.data
				.unwrap_or_default();
			let courses = service_parsed
				.courses
				.iter()
				.map(|course| entity::course::ActiveModel {
					id: ActiveValue::Set(course.id),
					name: ActiveValue::Set(course.full_name.clone()),
					colour: ActiveValue::Set(Some("blue".to_string())),
					icon: ActiveValue::NotSet,
				})
				.collect::<Vec<_>>();

			let state = app_handle.state::<DatabaseState>();
			let db = &state.0;
			let txn = db.begin().await.map_err(|error| error.to_string())?;
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
					.map_err(|error| error.to_string())?;
			}

			txn.commit().await.map_err(|error| error.to_string())?;
			Ok(())
		},
	)
	.await;

	let db = &state.0;
	entity::Course::find()
		.all(db)
		.await
		.map_err(|error| error.to_string())
		.map(|courses| courses.into_iter().collect())
}
