use serde::{Deserialize, Serialize};
use sqlx::Error;
use tauri::{http::HeaderMap, AppHandle};
use tauri_plugin_http::reqwest::{self};
use tauri_plugin_store::StoreExt;

use crate::{auth::auth_keys, database::DatabaseState, sql_query::SqlQuery, Course};

mod service_methods {
	pub const GET_NOTIFICATIONS: &str = "message_popup_get_popup_notifications";
	pub const GET_COURSES: &str = "theme_remui_get_myoverviewcourses";
}

#[derive(Serialize, Deserialize)]
struct ServiceMethod<'a> {
	index: u32,
	method_name: &'a str,
	args: ServiceMethodArgs<'a>,
}

#[derive(Serialize, Deserialize)]
struct ServiceMethodArgs<'a> {
	// these are named as they would be defined in the request
	limit: Option<u32>,
	offset: Option<u32>,
	useridto: Option<&'a str>,
	classification: Option<&'a str>,
	customfieldname: Option<&'a str>,
	customfieldvalue: Option<&'a str>,
	sort: Option<&'a str>,
}

impl<'a> ServiceMethod<'a> {
	fn new(index: u32, method_name: &'a str) -> Self {
		Self {
			index,
			method_name,
			args: ServiceMethodArgs {
				limit: None,
				offset: None,
				useridto: None,
				classification: None,
				customfieldname: None,
				customfieldvalue: None,
				sort: None,
			},
		}
	}

	fn to_string(&self) -> String {
		return serde_json::to_string(&self.args).unwrap();
	}

	fn with_limit(mut self, limit: u32) -> Self {
		self.args.limit = Some(limit);
		self
	}

	fn with_offset(mut self, offset: u32) -> Self {
		self.args.offset = Some(offset);
		self
	}

	fn with_user_id_to(mut self, useridto: &'a str) -> Self {
		self.args.useridto = Some(useridto);
		self
	}

	fn with_classification(mut self, classification: &'a str) -> Self {
		self.args.classification = Some(classification);
		self
	}

	fn with_custom_field_name(mut self, customfieldname: &'a str) -> Self {
		self.args.customfieldname = Some(customfieldname);
		self
	}

	fn with_custom_field_value(mut self, customfieldvalue: &'a str) -> Self {
		self.args.customfieldvalue = Some(customfieldvalue);
		self
	}

	fn with_sort(mut self, sort: &'a str) -> Self {
		self.args.sort = Some(sort);
		self
	}
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
) -> Result<Vec<Course>, Error> {
	let auth_store = app.store("auth").unwrap();
	let session_cookie = auth_store.get(auth_keys::MOODLE_SESSION).unwrap();
	let session_value = session_cookie.as_str().unwrap();
	let client = reqwest::Client::new();
	let service_method = ServiceMethod::new(0, service_methods::GET_COURSES)
		.with_classification("all")
		.with_sort("fullname");

	let host = auth_store.get(auth_keys::MOODLE_HOST).unwrap();
	let request = build_service_request(
		&client,
		&host.as_str().unwrap(),
		session_value,
		vec![service_method],
	)
	.unwrap();
	let response = client.execute(request).await.unwrap();
	let body = response.text().await.unwrap();
	let mut courses: Vec<Course> = serde_json::from_str(&body).unwrap();

	let pool = &state.0;
	let query = SqlQuery::new().pool(pool);
	// .insert_into(courses)
	// .await
	// .unwrap();

	return Ok(courses);
}

fn build_service_request(
	client: &reqwest::Client,
	host: &str,
	session_cookie: &str,
	service_methods: Vec<ServiceMethod>,
) -> Result<reqwest::Request, reqwest::Error> {
	let endpoint = format!("{host}/lib/ajax/service.php");
	let mut headers = HeaderMap::new();
	headers.insert("Accept", "application/json".parse().unwrap());
	headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36".parse().unwrap());
	headers.insert(
		"Cookie",
		format!("MoodleSession={session_cookie}").parse().unwrap(),
	);

	let request = client
		.post(endpoint)
		.headers(headers)
		.body(serde_json::to_string(&service_methods).unwrap())
		.build();

	return request;
}
