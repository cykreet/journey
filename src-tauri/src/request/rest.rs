use ::serde::Deserialize;
use entity::section_module::SectionModuleType;
use tauri::http::{HeaderMap, HeaderValue};
use tauri_plugin_http::reqwest::{self};

pub mod rest_functions {
	pub const GET_USER_COURSES: &str = "core_enrol_get_users_courses";
	pub const GET_COURSE_CONTENT: &str = "core_course_get_contents";
}

#[derive(Debug, Deserialize)]
pub struct RestCourseSection {
	pub id: i32,
	pub name: String,
	#[serde(rename = "section")]
	pub rank: i32,
	pub modules: Vec<RestCourseSectionModule>,
}

#[derive(Debug, Deserialize)]
pub struct RestCourseSectionModule {
	pub id: i32,
	pub name: String,
	pub description: Option<String>,
	#[serde(rename = "modname")]
	pub module_type: SectionModuleType,
	pub contents: Option<Vec<RestCourseSectionModuleContent>>,
	#[serde(rename = "contentsinfo")]
	pub contents_info: Option<RestCourseSectionModuleContentInfo>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum RestCourseSectionModuleContentType {
	#[serde(rename = "file")]
	File,
	#[serde(rename = "content")]
	Content,
}

#[derive(Debug, Deserialize)]
pub struct RestCourseSectionModuleContent {
	#[serde(rename = "filename")]
	pub file_name: String,
	#[serde(rename = "filepath")]
	pub file_path: String,
	#[serde(rename = "fileurl")]
	pub file_url: Option<String>,
	#[serde(rename = "timemodified")]
	pub time_modified: u64,
	#[serde(rename = "mimetype")]
	pub mime_type: Option<String>,
	#[serde(rename = "isexternalfile")]
	pub is_external_file: Option<bool>,
	#[serde(rename = "type")]
	pub content_type: RestCourseSectionModuleContentType,
	#[serde(rename = "content")]
	pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RestCourseSectionModuleContentInfo {
	#[serde(rename = "filescount")]
	pub files_count: u32,
	#[serde(rename = "mimetypes")]
	pub mime_types: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct RestCourseSectionModuleStructureItem {
	pub title: String,
	pub href: String,
	#[serde(rename = "subitems")]
	pub sub_items: Option<Vec<RestCourseSectionModuleStructureItem>>,
}

#[derive(Debug, Deserialize)]
pub struct RestCourse {
	pub id: i32,
	#[serde(rename = "fullname")]
	pub full_name: String,
}

fn build_rest_request(
	client: &reqwest::Client,
	host: &str,
	ws_token: &str,
	form: &mut std::collections::HashMap<String, String>,
) -> Result<reqwest::Request, Box<dyn std::error::Error>> {
	let endpoint = format!("{host}/webservice/rest/server.php");
	let mut headers = HeaderMap::new();
	headers.insert("Accept", HeaderValue::from_static("application/json"));
	headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (iPhone; CPU iPhone OS 19_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 MoodleMobile 5.0.0 (50003)"));
	headers.insert(
		"Origin",
		HeaderValue::from_static("moodleappfs://localhost"),
	);

	form.insert("moodlewsrestformat".to_string(), "json".to_string());
	form.insert("moodlewssettinglang".to_string(), "en".to_string());
	form.insert("wstoken".to_string(), ws_token.to_string());

	Ok(client.post(endpoint).headers(headers).form(&form).build()?)
}

pub fn get_user_courses_request(
	user_id: u32,
	host: &str,
	ws_token: &str,
) -> Result<reqwest::Request, Box<dyn std::error::Error>> {
	let form = &mut std::collections::HashMap::new();
	form.insert("userid".to_string(), user_id.to_string());
	form.insert(
		"wsfunction".to_string(),
		rest_functions::GET_USER_COURSES.to_string(),
	);

	build_rest_request(&reqwest::Client::new(), host, ws_token, form)
}

pub fn get_course_sections_request(
	course_id: i32,
	client: &reqwest::Client,
	host: &str,
	ws_token: &str,
) -> Result<reqwest::Request, Box<dyn std::error::Error>> {
	let form = &mut std::collections::HashMap::new();
	form.insert(
		"wsfunction".to_string(),
		rest_functions::GET_COURSE_CONTENT.to_string(),
	);
	form.insert(
		"options[0][name]".to_string(),
		"excludecontents".to_string(),
	);
	form.insert("courseid".to_string(), course_id.to_string());
	form.insert("options[0][value]".to_string(), "1".to_string());

	build_rest_request(&client, host, ws_token, form)
}

/// this endpoint returns data similar to the course content endpoint (used for sections and modules),
/// but it also includes only the specified module's content
pub fn get_sections_with_model_content(
	course_id: i32,
	module_id: i32,
	client: &reqwest::Client,
	host: &str,
	ws_token: &str,
) -> Result<reqwest::Request, Box<dyn std::error::Error>> {
	let form = &mut std::collections::HashMap::new();
	form.insert(
		"wsfunction".to_string(),
		rest_functions::GET_COURSE_CONTENT.to_string(),
	);
	form.insert("courseid".to_string(), course_id.to_string());
	form.insert(
		"options[0][name]".to_string(),
		"includestealthmodules".to_string(),
	);
	form.insert("options[0][value]".to_string(), "1".to_string());
	form.insert("options[1][name]".to_string(), "cmid".to_string());
	form.insert("options[1][value]".to_string(), module_id.to_string());

	build_rest_request(&client, host, ws_token, form)
}
