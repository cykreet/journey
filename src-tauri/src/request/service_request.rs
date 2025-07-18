use serde::{Deserialize, Serialize};
use tauri::http::{HeaderMap, HeaderValue};
use tauri_plugin_http::reqwest::{self};

// todo: probably gonna need more robust methods for different versions/platforms
pub mod service_methods {
	pub const GET_COURSES: &str = "theme_remui_get_myoverviewcourses";
	pub const GET_COURSE_STATE: &str = "core_courseformat_get_state";
	// pub const GET_NOTIFICATIONS: &str = "message_popup_get_popup_notifications";
	// pub const SESSION_TOUCH: &str = "core_session_touch";
}

#[derive(Debug, Deserialize)]
pub struct ServiceResponse<T> {
	pub data: Option<T>,
	pub error: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct ServiceMethod<'a> {
	index: i32,
	methodname: &'a str,
	args: ServiceMethodArgs<'a>,
}

#[derive(Serialize, Deserialize)]
pub struct ServiceMethodArgs<'a> {
	#[serde(skip_serializing_if = "Option::is_none")]
	limit: Option<i32>,

	#[serde(skip_serializing_if = "Option::is_none")]
	offset: Option<i32>,

	#[serde(rename = "useridto")]
	#[serde(skip_serializing_if = "Option::is_none")]
	user_id_to: Option<&'a str>,

	#[serde(skip_serializing_if = "Option::is_none")]
	classification: Option<&'a str>,

	#[serde(rename = "customfieldname")]
	#[serde(skip_serializing_if = "Option::is_none")]
	custom_field_name: Option<&'a str>,

	#[serde(rename = "customfieldvalue")]
	#[serde(skip_serializing_if = "Option::is_none")]
	custom_field_value: Option<&'a str>,

	#[serde(skip_serializing_if = "Option::is_none")]
	sort: Option<&'a str>,

	#[serde(rename = "courseid")]
	#[serde(skip_serializing_if = "Option::is_none")]
	course_id: Option<i32>,
}

impl<'a> ServiceMethod<'a> {
	pub fn new(index: i32, methodname: &'a str) -> Self {
		Self {
			index,
			methodname,
			args: ServiceMethodArgs {
				limit: None,
				offset: None,
				user_id_to: None,
				classification: None,
				custom_field_name: None,
				custom_field_value: None,
				sort: None,
				course_id: None,
			},
		}
	}

	pub fn with_limit(mut self, limit: i32) -> Self {
		self.args.limit = Some(limit);
		self
	}

	pub fn with_offset(mut self, offset: i32) -> Self {
		self.args.offset = Some(offset);
		self
	}

	pub fn with_sort(mut self, sort: &'a str) -> Self {
		self.args.sort = Some(sort);
		self
	}

	pub fn with_classification(mut self, classification: &'a str) -> Self {
		self.args.classification = Some(classification);
		self
	}

	// fn with_user_id_to(mut self, useridto: &'a str) -> Self {
	// 	self.args.useridto = Some(useridto);
	// 	self
	// }

	// fn with_custom_field_name(mut self, customfieldname: &'a str) -> Self {
	// 	self.args.customfieldname = Some(customfieldname);
	// 	self
	// }

	// fn with_custom_field_value(mut self, customfieldvalue: &'a str) -> Self {
	// 	self.args.customfieldvalue = Some(customfieldvalue);
	// 	self
	// }

	pub fn with_course_id(mut self, course_id: i32) -> Self {
		self.args.course_id = Some(course_id);
		self
	}
}

pub fn build_service_request(
	client: &reqwest::Client,
	host: &str,
	session_cookie: &str,
	session_key: &str,
	service_methods: Vec<ServiceMethod>,
) -> Result<reqwest::Request, Box<dyn std::error::Error>> {
	let endpoint = format!("https://{host}/lib/ajax/service.php?sesskey={session_key}");
	let mut headers = HeaderMap::new();
	headers.insert("Accept", HeaderValue::from_static("application/json"));
	headers.insert("Content-Type", HeaderValue::from_static("application/json"));
	headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"));
	headers.insert(
		"Cookie",
		HeaderValue::from_str(&format!("MoodleSession={session_cookie}")).unwrap(),
	);

	let body = serde_json::to_string(&service_methods)?;
	Ok(client.post(endpoint).headers(headers).body(body).build()?)
}
