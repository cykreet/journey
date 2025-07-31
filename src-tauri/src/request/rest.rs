use ::serde::{Deserialize, Serialize};
use tauri::http::{HeaderMap, HeaderValue};
use tauri_plugin_http::reqwest::{self};

pub mod rest_functions {
	pub const GET_COURSES_BY_FIELD: &str = "core_course_get_courses_by_field";
}

#[derive(Debug, Deserialize)]
pub struct RestResponse<T> {
	pub responses: Vec<FunctionResponse<T>>,
}

#[derive(Debug, Deserialize)]
pub struct FunctionResponse<T> {
	pub error: bool,
	pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestRequestBody {
	pub requests: Vec<RestFunctionCall>,
	#[serde(rename = "wsfunction")]
	pub ws_function: String,
	#[serde(rename = "wstoken")]
	pub ws_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestFunctionCall {
	pub function: String,
	pub args: serde_json::Value,
	#[serde(skip_serializing_if = "Option::is_none", rename = "settingfileurl")]
	pub setting_file_url: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none", rename = "settingfilter")]
	pub setting_filter: Option<String>,
}

pub fn build_rest_request(
	client: &reqwest::Client,
	host: &str,
	token: &str,
	functions: Vec<RestFunctionCall>,
) -> Result<reqwest::Request, Box<dyn std::error::Error>> {
	let endpoint = format!("https://{host}/webservice/rest/server.php?moodlewsrestformat=json&wsfunction=tool_mobile_call_external_functions");

	let mut headers = HeaderMap::new();
	headers.insert("Accept", HeaderValue::from_static("application/json"));
	headers.insert(
		"Content-Type",
		HeaderValue::from_static("application/x-www-form-urlencoded"),
	);
	headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (iPhone; CPU iPhone OS 19_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 MoodleMobile 5.0.0 (50003)"));
	headers.insert(
		"Origin",
		HeaderValue::from_static("moodleappfs://localhost"),
	);

	let body = RestRequestBody {
		requests: functions,
		ws_function: "tool_mobile_call_external_functions".to_string(),
		ws_token: token.to_string(),
	};

	let body_encoded = serde_urlencoded::to_string(&body)
		.map_err(|e| format!("Failed to serialize request body: {}", e))?;
	Ok(
		client
			.post(endpoint)
			.headers(headers)
			.form(&body_encoded)
			.build()?,
	)
}
