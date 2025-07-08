use std::vec;

use serde::Deserialize;
use sqlx::types::Json;
use tauri::{AppHandle, Manager};
use tauri_plugin_http::reqwest;
use tauri_plugin_store::StoreExt;

use crate::{
	auth::auth_keys,
	database::DatabaseState,
	entities::{ContentType, Course, CourseSection, CourseSectionItem},
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
	fullname: String,
}

#[derive(Default, Deserialize)]
struct ServiceCourseState {
	section: Vec<ServiceCourseStateSection>,
	cm: Vec<ServiceCourseStateModule>,
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
	sectionid: String,
	// describes the type of the module, i.e. forum, page, etc
	module: String,
}

#[tauri::command]
#[specta::specta]
pub async fn get_user_course(
	state: tauri::State<'_, DatabaseState>,
	course_id: u32,
) -> Result<Course, String> {
	let pool = &state.0;
	// todo: join course sections
	let course = SqlQuery::new()
		.pool(&pool)
		.select_where::<Course>("id = ?", &vec![course_id.to_string()])
		.await
		.map_err(|error| error.to_string())?;
	Ok(course.first().unwrap().clone())
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

				return Err(message);
			}

			let body = response.text().await.unwrap();
			if body.contains("errorcode") {
				let message = format!("Could not get user courses: {}", body);
				return Err(message);
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
							.cm
							.iter()
							.filter(|cm| cm.sectionid == section.id)
							.map(|activity| CourseSectionItem {
								id: activity.id.parse().unwrap(),
								name: activity.name.clone(),
								// todo: map from module type
								content_type: ContentType::Resource,
							})
							.collect::<Vec<_>>(),
					),
				})
				.collect();

			let state = app_handle.state::<DatabaseState>();
			let pool = &state.0;
			SqlQuery::new()
				.pool(&pool)
				.insert_into(&sections)
				.await
				.map_err(|error| error.to_string())?;
			Ok(())
		},
	)
	.await;

	let pool = &state.0;
	let sections = SqlQuery::new()
		.pool(&pool)
		.select_where::<CourseSection>("course_id = ?", &vec![course_id.to_string()])
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

				return Err(message);
			}

			let body = response.text().await.unwrap();
			if body.contains("errorcode") {
				let message = format!("Could not get user courses: {}", body);
				return Err(message);
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
					name: course.fullname.clone(),
					colour: None,
					icon: None,
				})
				.collect();

			let state = app_handle.state::<DatabaseState>();
			let pool = &state.0;
			SqlQuery::new()
				.pool(&pool)
				.insert_into(&courses)
				.await
				.map_err(|error| error.to_string())?;

			Ok(())
		},
	)
	.await;

	let pool = &state.0;
	let courses = SqlQuery::new()
		.pool(&pool)
		.select::<Course>()
		.await
		.map_err(|error| error.to_string())?;
	Ok(courses)
}
