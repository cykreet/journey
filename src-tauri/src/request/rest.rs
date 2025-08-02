use ::serde::{Deserialize, Serialize};
use tauri::http::{HeaderMap, HeaderValue};
use tauri_plugin_http::reqwest::{self};

pub mod rest_functions {
	pub const GET_USER_COURSES: &str = "core_enrol_get_users_courses";
	// pub const GET_COURSES_BY_FIELD: &str = "core_course_get_courses_by_field";
	pub const GET_COURSE_SECTIONS: &str = "core_course_get_contents";
}

#[derive(Debug, Deserialize)]
pub struct RestResponse {
	pub responses: Vec<FunctionResponse>,
}

#[derive(Debug, Deserialize)]
pub struct FunctionResponse {
	pub error: bool,
	pub data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestRequestBody {
	pub requests: String,
	#[serde(rename = "wsfunction")]
	pub ws_function: String,
	#[serde(rename = "wstoken")]
	pub ws_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestFunctionCall {
	pub function: String,
	pub args: serde_json::Value,
	#[serde(rename = "settingfileurl")]
	pub setting_file_url: u32,
	#[serde(rename = "settingfilter")]
	pub setting_filter: u32,
}

#[derive(Debug, Deserialize)]
pub struct RestCourseSection {
	pub id: i32,
	pub name: String,
	#[serde(rename = "section")]
	pub number: i32,
	pub modules: Vec<RestCourseSectionModule>,
}

#[derive(Debug, Deserialize)]
pub struct RestCourseSectionModule {
	pub id: i32,
	pub name: String,
	pub description: Option<String>,
	pub contents: Option<RestCourseSectionModuleContent>,
}

#[derive(Debug, Deserialize)]
pub struct RestCourseSectionModuleContent {
	#[serde(rename = "filename")]
	pub file_name: Option<String>,
	#[serde(rename = "filepath")]
	pub file_path: Option<String>,
	#[serde(rename = "fileurl")]
	pub file_url: Option<String>,
	#[serde(rename = "timemodified")]
	pub time_modified: u64,
	#[serde(rename = "mimetype")]
	pub mime_type: Option<String>,
	#[serde(rename = "isexternalfile")]
	pub is_external_file: bool,
	#[serde(rename = "type")]
	pub content_type: String,
	#[serde(rename = "content")]
	pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RestCourse {
	pub id: i32,
	#[serde(rename = "fullname")]
	pub full_name: String,
}

// rest_functions::GET_COURSES_BY_FIELD
// https://github.com/moodlehq/moodleapp/blob/main/src/core/features/courses/services/courses.ts#L480
#[derive(Debug, Deserialize)]
// untagged means enum variants are not relevant to deserialisation, only the contained types
// https://serde.rs/enum-representations.html#untagged
#[serde(untagged)]
pub enum GetCoursesFunctionData {
	Courses(Vec<RestCourse>),
}

// rest_functions::GET_COURSE_SECTIONS
// https://github.com/moodlehq/moodleapp/blob/main/src/core/features/course/services/course.ts#L895
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetCourseSectionsFunctionData {
	Sections(Vec<RestCourseSection>),
}

pub fn build_rest_request(
	client: &reqwest::Client,
	host: &str,
	token: &str,
	functions: Vec<RestFunctionCall>,
) -> Result<reqwest::Request, Box<dyn std::error::Error>> {
	let endpoint = format!("{host}/webservice/rest/server.php");
	let mut headers = HeaderMap::new();
	headers.insert("Accept", HeaderValue::from_static("application/json"));
	headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (iPhone; CPU iPhone OS 19_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 MoodleMobile 5.0.0 (50003)"));
	headers.insert(
		"Origin",
		HeaderValue::from_static("moodleappfs://localhost"),
	);

	let mut form = std::collections::HashMap::new();
	form.insert("moodlewsrestformat".to_string(), "json".to_string());
	form.insert(
		"wsfunction".to_string(),
		"tool_mobile_call_external_functions".to_string(),
	);
	form.insert("moodlewssettinglang".to_string(), "en".to_string());
	form.insert("wstoken".to_string(), token.to_string());

	for (i, func) in functions.iter().enumerate() {
		form.insert(format!("requests[{}][function]", i), func.function.clone());
		form.insert(
			format!("requests[{}][settingfileurl]", i),
			func.setting_file_url.to_string(),
		);
		form.insert(
			format!("requests[{}][settingfilter]", i),
			func.setting_filter.to_string(),
		);
		form.insert(format!("requests[{}][arguments]", i), func.args.to_string());
	}

	Ok(client.post(endpoint).headers(headers).form(&form).build()?)
}

// pub fn get_courses_by_field(field: &str, value: &str) -> RestFunctionCall {
// 	RestFunctionCall {
// 		function: rest_functions::GET_COURSES_BY_FIELD.to_string(),
// 		args: serde_json::json!({ "field": field, "value": value }),
// 		setting_file_url: 1,
// 		setting_filter: 1,
// 	}
// }

pub fn get_user_courses(user_id: u32) -> RestFunctionCall {
	RestFunctionCall {
		function: rest_functions::GET_USER_COURSES.to_string(),
		args: serde_json::json!({ "userid": user_id, "returnusercount": "0" }),
		setting_file_url: 1,
		setting_filter: 1,
	}
}

pub fn get_course_sections(course_id: i32, exclude_contents: bool) -> RestFunctionCall {
	RestFunctionCall {
		function: rest_functions::GET_COURSE_SECTIONS.to_string(),
		args: serde_json::json!({ "courseid": course_id, "options": [{ "name": "excludecontents", "value": exclude_contents }] }),
		setting_file_url: 1,
		setting_filter: 1,
	}
}
