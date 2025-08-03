use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Emitter, Manager, Url, WebviewWindowBuilder};
use tauri_plugin_http::reqwest;
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;

pub mod auth_keys {
	pub const MOODLE_HOST: &str = "moodle_host";
	pub const WS_TOKEN: &str = "ws_token";
	pub const PASSPORT: &str = "passport";
	pub const USER_ID: &str = "user_id";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum AuthStatus {
	Failed,
	Success,
	Aborted,
	Pending,
}

impl Default for AuthStatus {
	fn default() -> Self {
		AuthStatus::Pending
	}
}

#[derive(Default, Deserialize)]
struct RestSiteInfo {
	#[serde(rename = "sitename")]
	pub site_name: String,
	#[serde(rename = "userid")]
	pub user_id: u32,
}

// auth state represents the current status of the auth process, i.e whether
// the user is currently being authenticated or not, this used for handling
// window close events to abort authentication
#[derive(Default)]
pub struct AuthState {
	pub auth_status: AuthStatus,
}

#[tauri::command]
#[specta::specta]
pub async fn open_login_window(app: AppHandle, host: &str) -> Result<(), String> {
	if host.is_empty() {
		return Err("invalid domain".to_string());
	}

	// we use the mobile app login endpoint, which gives us a token that we can use to authenticate
	// this token should last a lot longer than normal session cookies.
	let mut login_url = Url::parse(&format!("{}{}", host, "/admin/tool/mobile/launch.php")).unwrap();
	let passport: f64 = rand::rng().random_range(0.0..1000.0);
	login_url.set_query(Some(&format!(
		"service=moodle_mobile_app&passport={}&urlscheme=moodlemobile&lang=en",
		passport
	)));

	let auth_state = app.state::<Mutex<AuthState>>();
	let mut auth_state = auth_state.lock().await;
	auth_state.auth_status = AuthStatus::Pending;

	let validate_response = reqwest::Client::new()
		.post(&format!(
			"{}/lib/ajax/service.php?info=tool_mobile_get_public_config&lang=en",
			host
		))
		.send()
		.await
		.map_err(|_| "invalid domain".to_string())?;

	if validate_response.status().is_success() == false {
		return Err("invalid moodle instance".to_string());
	}

	let host = host.to_string();
	let app_handle = app.clone();
	let window_label = "login";
	WebviewWindowBuilder::new(&app, window_label, tauri::WebviewUrl::External(login_url))
		.on_navigation(move |url| {
			// once we're navigated to the moodlemobile:// url, we can extract the token
			// in the form of moodlemobile://token=ws_token
			if url.scheme() != "moodlemobile" {
				return true;
			}

			let window = app_handle.get_webview_window(window_label).unwrap();
			let store = window.store("store.json");
			if store.is_err() {
				println!("could not open store");
				window.close().unwrap();
				return false;
			}

			let store = store.unwrap();
			let auth_state = app_handle.state::<Mutex<AuthState>>();
			let mut auth_state = tauri::async_runtime::block_on(auth_state.lock());

			// token is base64 encoded, in the format signature:::ws_token:::private_token
			// signature isn't super relevant to us, but it's a hash of the site url and passport
			let token = url.as_str().split("token=").nth(1).unwrap_or_default();
			if token.is_empty() {
				println!("No token found in URL");
				auth_state.auth_status = AuthStatus::Failed;

				window.emit("moodle_auth", &auth_state.auth_status).unwrap();
				window.close().unwrap();
				return false;
			}

			let token = base64::prelude::BASE64_STANDARD.decode(token).unwrap();
			let token = String::from_utf8(token).unwrap();
			let token_parts: Vec<&str> = token.split(":::").collect();
			if token_parts.len() < 2 {
				println!("Invalid token format");
				auth_state.auth_status = AuthStatus::Failed;

				window.emit("moodle_auth", &auth_state.auth_status).unwrap();
				window.close().unwrap();
				return false;
			}

			println!("Token: {}", token_parts[1]);

			let host = host.clone();
			tauri::async_runtime::block_on(async move {
				let site_info_response = reqwest::Client::new()
					.get(&format!("{}/webservice/rest/server.php", host))
					.query(&[
						("moodlewsrestformat", "json"),
						("wsfunction", "core_webservice_get_site_info"),
						("wstoken", token_parts[1]),
						("moodlewssettinglang", "en"),
						("moodlewssettingfileurl", "true"),
						("moodlewssettingfilter", "true"),
					])
					.send()
					.await;

				if site_info_response.is_err() {
					println!(
						"Failed to fetch site info: {}",
						site_info_response.err().unwrap()
					);
					auth_state.auth_status = AuthStatus::Failed;
					window.emit("moodle_auth", &auth_state.auth_status).unwrap();
					window.close().unwrap();
					return false;
				}

				let response_body = site_info_response.unwrap().text().await;
				if response_body.is_err() {
					println!(
						"Failed to read response body: {}",
						response_body.err().unwrap()
					);
					auth_state.auth_status = AuthStatus::Failed;
					window.emit("moodle_auth", &auth_state.auth_status).unwrap();
					window.close().unwrap();
					return false;
				}

				let site_info: RestSiteInfo = serde_json::from_str(&response_body.unwrap())
					.map_err(|e| {
						println!("Failed to parse site info: {}", e);
						"Failed to parse site info".to_string()
					})
					.unwrap();

				// todo: store user data in separate table with data like enrolled courses
				// probably means we also have to encrypt course data
				store.set(auth_keys::USER_ID, site_info.user_id.to_string());
				store.set(auth_keys::MOODLE_HOST, host);
				store.set(auth_keys::WS_TOKEN, token_parts[1]);
				store.set(auth_keys::PASSPORT, passport);

				auth_state.auth_status = AuthStatus::Success;
				window.emit("moodle_auth", &auth_state.auth_status).unwrap();
				window
					.get_webview_window("main")
					.unwrap()
					.set_focus()
					.unwrap();
				window.close().unwrap();
				return true;
			});

			return true;
		})
		.build()
		.unwrap();

	Ok(())
}
