use std::vec;

use serde::Deserialize;
use sqlx::types::Json;
use tauri::{AppHandle, Manager};
use tauri_plugin_http::reqwest;
use tauri_plugin_store::StoreExt;

use crate::{
	auth::auth_keys,
	database::DatabaseState,
	entities::{ContentType, Course, CourseSection, CourseSectionItem, TableLike},
	request::service_request::{
		build_service_request, service_methods, ServiceMethod, ServiceResponse,
	},
	sql_query::SqlQuery,
	store_keys,
	sync::{self},
};

#[derive(Default, Deserialize)]
struct ServiceCourses {
	courses: Vec<ServiceCourse>,
}

#[derive(Deserialize)]
struct ServiceCourse {
	id: u32,
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
	module: String,
}

#[tauri::command]
#[specta::specta]
pub async fn get_user_course(
	state: tauri::State<'_, DatabaseState>,
	course_id: u32,
) -> Result<Course, String> {
	// todo: join course sections, joins are not currently supported by SqlQuery
	// and would be a decent amount of work outside of just writing the query
	let pool = &state.0;
	let course = SqlQuery::new()
		.select(Course::table_name())
		.where_column("id", course_id.to_string())
		.fetch_one::<Course, _>(pool)
		.await
		.map_err(|error| error.to_string())?;
	Ok(course)
}

#[tauri::command]
#[specta::specta]
pub async fn get_course_sections(
	app: AppHandle,
	state: tauri::State<'_, DatabaseState>,
	course_id: u32,
) -> Result<Vec<CourseSection>, String> {
	sync::revalidate_task(
		&app,
		format!("get_course_sections_{}", course_id).as_str(),
		"Get course sections",
		async move |app_handle| {
			let auth_store = app_handle.store(store_keys::AUTH).unwrap();
			let client = reqwest::Client::new();
			let service_method =
				ServiceMethod::new(0, service_methods::GET_COURSE_STATE).with_courseid(course_id);

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
			// individual section items or "cm" (course modules) are stored in a separate array from
			// the sections, so we need to merge them (see ServiceCourseState)
			let course_state = serde_json::from_str::<ServiceCourseState>(&service_parsed).unwrap();
			let sections = course_state
				.section
				.iter()
				.map(|section| CourseSection {
					id: section.id.parse().unwrap(),
					name: section.title.clone(),
					course_id: course_id,
					items: Json::from(
						course_state
							.module
							.iter()
							.filter(|module| module.section_id == section.id)
							.map(|module| CourseSectionItem {
								id: module.id.parse().unwrap(),
								name: module.name.clone(),
								updated_at: None,
								// todo: map from module type
								content_type: ContentType::Page,
							})
							.collect::<Vec<_>>(),
					),
				})
				.collect::<Vec<_>>();

			let state = app_handle.state::<DatabaseState>();
			let mut transaction = state.0.begin().await?;
			SqlQuery::new()
				.insert_into(&sections)
				.execute(transaction.as_mut())
				.await
				.map_err(|error| error.to_string())?;

			transaction.commit().await?;
			Ok(())
		},
	)
	.await;

	let pool = &state.0;
	let sections = SqlQuery::new()
		.select(CourseSection::table_name())
		.where_column("course_id", course_id.to_string())
		.fetch_all::<CourseSection, _>(pool)
		.await
		.map_err(|error| error.to_string())?;
	Ok(sections)
}

#[tauri::command]
#[specta::specta]
pub async fn get_user_courses(
	app: AppHandle,
	state: tauri::State<'_, DatabaseState>,
) -> Result<Vec<Course>, String> {
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
				.map(|course| Course {
					id: course.id,
					name: course.full_name.clone(),
					colour: None,
					icon: None,
				})
				.collect();

			let state = app_handle.state::<DatabaseState>();
			let mut transaction = state.0.begin().await?;
			SqlQuery::new()
				.insert_into(&courses)
				.execute(transaction.as_mut())
				.await?;

			transaction.commit().await?;
			Ok(())
		},
	)
	.await;

	let pool = &state.0;
	let courses = SqlQuery::new()
		.select(Course::table_name())
		.fetch_all::<Course, _>(pool)
		.await
		.map_err(|error| error.to_string())?;
	Ok(courses)
}
