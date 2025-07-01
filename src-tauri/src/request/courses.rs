use select::{
	document::Document,
	predicate::{Attr, Class, Element, Name, Predicate},
};
use serde::Deserialize;
use tauri::AppHandle;
use tauri_plugin_http::reqwest;
use tauri_plugin_store::StoreExt;

use crate::{
	auth::auth_keys,
	database::DatabaseState,
	entities::Course,
	request::service_request::{
		build_service_request, service_methods, ServiceMethod, ServiceResponse,
	},
	sql_query::SqlQuery,
	store_keys,
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

#[tauri::command]
#[specta::specta]
pub async fn get_user_course(
	state: tauri::State<'_, DatabaseState>,
	course_id: u32,
) -> Result<Course, String> {
	let pool = &state.0;
	let course = SqlQuery::new()
		.pool(&pool)
		.select_where::<Course>("id = ?", &vec![course_id.to_string()])
		.await
		.map_err(|error| error.to_string())?;
	Ok(course.first().unwrap().clone())
}

#[tauri::command]
#[specta::specta]
pub async fn get_user_courses(
	app: AppHandle,
	state: tauri::State<'_, DatabaseState>,
) -> Result<Vec<Course>, String> {
	let courses = sync_user_courses(&app, &state)
		.await
		.map_err(|error| error.to_string())?;
	Ok(courses)
}

async fn sync_user_courses(
	app: &AppHandle,
	state: &tauri::State<'_, DatabaseState>,
) -> Result<Vec<Course>, String> {
	let auth_store = app.store(store_keys::AUTH).unwrap();
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

	let pool = &state.0;
	SqlQuery::new()
		.pool(&pool)
		.insert_into(&courses)
		.await
		.unwrap();

	Ok(courses)
}

// async fn sync_course_items() {
// 	let document = Document::from("epic time");
// 	let item_sections = document.find(
// 		Name("nav")
// 			.and(Class("courseindex"))
// 			.descendant(Attr("data-for", "section")),
// 	);

// 	for section in item_sections {
// 		let section_title = section
// 			.find(Attr("data-for", "section_item").descendant(Name("a")))
// 			.next()
// 			.unwrap();

// 		let section_links = section.find(Attr("data-for", "cmlist").descendant(Name("a")));
// 	}
// }
