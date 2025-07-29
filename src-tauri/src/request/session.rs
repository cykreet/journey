use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{webview::Cookie, Url};
use tauri_plugin_http::reqwest;

use crate::request::service_request::{
	build_service_request, service_methods, ServiceMethod, ServiceResponse,
};

#[derive(Deserialize, Serialize, Type, Clone)]
pub enum SessionStatus {
	Valid,
	Expired,
	Invalid,
}

#[derive(Default, Deserialize)]
struct ServiceRemainingTime {
	#[serde(rename = "userid")]
	user_id: u32,
	#[serde(rename = "timeremaining")]
	remaining_time: u64,
}

pub async fn get_remaining_time(
	host: Url,
	session_cookie: &str,
	session_key: &str,
) -> Result<u64, String> {
	let client = reqwest::Client::new();
	let service_method = ServiceMethod::new(0, service_methods::GET_REMAINING_TIME);
	let request = build_service_request(
		&client,
		&host.host().unwrap().to_string(),
		&session_cookie,
		&session_key,
		vec![service_method],
	)
	.unwrap();

	let response = client.execute(request).await.map_err(|e| e.to_string())?;
	if response.status().is_success() == false {
		return Err(format!(
			"Failed to get remaining time: {}",
			response.status()
		));
	}

	let body = response.text().await.map_err(|e| e.to_string())?;
	let service_body = serde_json::from_str::<Vec<ServiceResponse<ServiceRemainingTime>>>(&body)
		.map_err(|e| e.to_string())?;
	let body_parsed = service_body.into_iter().next().unwrap();
	let response_data = body_parsed.data.ok_or("No data in response")?;
	if let Some(exception) = body_parsed.exception {
		if exception.error_code == "invalidsesskey" {
			return Ok(0);
		}

		return Err(format!(
			"Service exception: {} (code: {})",
			exception.message, exception.error_code
		));
	}

	Ok(response_data.remaining_time)
}

pub async fn touch_session(
	host: Url,
	session_cookie: &str,
	session_key: &str,
) -> Result<(), String> {
	let client = reqwest::Client::new();
	let service_method = ServiceMethod::new(0, service_methods::SESSION_TOUCH);
	let request = build_service_request(
		&client,
		&host.host().unwrap().to_string(),
		&session_cookie,
		&session_key,
		vec![service_method],
	)
	.unwrap();

	let response = client.execute(request).await.map_err(|e| e.to_string())?;
	if response.status().is_success() == false {
		return Err(format!("Failed to touch session: {}", response.status()));
	}

	let body = response.text().await.map_err(|e| e.to_string())?;
	let service_body =
		serde_json::from_str::<Vec<ServiceResponse<()>>>(&body).map_err(|e| e.to_string())?;
	let body_parsed = service_body.into_iter().next().unwrap();
	if let Some(exception) = body_parsed.exception {
		return Err(format!(
			"Service exception: {} (code: {})",
			exception.message, exception.error_code
		));
	}

	Ok(())
}

pub async fn fetch_session_key(host: Url, session_cookie: String) -> String {
	const SESSION_KEY: &str = r#""sesskey":""#;
	const KEY_LENGTH: usize = SESSION_KEY.len();

	let response = reqwest::Client::new()
		.get(host)
		.header("Cookie", format!("MoodleSession={}", session_cookie))
		.send()
		.await
		.unwrap();

	let body = response.text().await.unwrap();
	let session_key_start = body.find(SESSION_KEY).unwrap() + KEY_LENGTH;
	let session_key_end = body[session_key_start..].find('"').unwrap() + session_key_start;
	let session_key = &body[session_key_start..session_key_end];
	session_key.to_string()
}

pub fn find_session_cookie<'a>(cookies: &'a Vec<Cookie<'static>>) -> Option<&'a Cookie<'a>> {
	cookies
		.iter()
		.find(|cookie| cookie.name() == "MoodleSession")
}
