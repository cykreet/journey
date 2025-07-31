use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{webview::Cookie, AppHandle, Emitter, Manager, Url, WebviewWindowBuilder};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;

pub mod auth_keys {
	pub const INITIAL_SESSION: &str = "initial_session";
	pub const MOODLE_HOST: &str = "moodle_host";
	pub const WS_TOKEN: &str = "ws_token";
	pub const PASSPORT: &str = "passport";
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

// auth state represents the current status of the auth process, i.e whether
// the user is currently being authenticated or not, this used for handling
// window close events to abort authentication
#[derive(Default)]
pub struct AuthState {
	pub auth_status: AuthStatus,
}

#[tauri::command]
#[specta::specta]
pub async fn open_login_window(app: AppHandle, domain: &str) -> Result<(), String> {
	if domain.is_empty() {
		return Err("invalid domain".to_string());
	}

	// we use the mobile app login endpoint, which gives us a token that we can use to authenticate.
	// this token should last a lot longer than normal session cookies.
	let mut login_url =
		Url::parse(&format!("{}{}", domain, "/admin/tool/mobile/launch.php")).unwrap();
	let passport: f64 = rand::rng().random_range(0.0..1000.0);
	login_url.set_query(Some(&format!(
		"service=moodle_mobile_app&passport={}&urlscheme=moodlemobile&lang=en",
		passport
	)));

	let store = app.store("store.json").unwrap();
	store.delete(auth_keys::INITIAL_SESSION);

	let auth_state = app.state::<Mutex<AuthState>>();
	let mut auth_state = auth_state.lock().await;
	auth_state.auth_status = AuthStatus::Pending;

	// todo: replace with service method request to global site info
	// let validate_response = reqwest::Client::new()
	// 	.get(window_url.clone())
	// 	.send()
	// 	.await
	// .unwrap();

	// let mut validate_cookies = validate_response.cookies();
	// let has_moodle_session = validate_cookies.any(|cookie| cookie.name() == "MoodleSession");
	// the session cookie should be present on all requests relevant to a (possibly unmodified)
	// moodle instance, if it's not present and we're on the requested url, it's probably not a
	// valid instance.
	// if validate_response.status().is_success() == false || has_moodle_session == false {
	// 	return Err("invalid instance".to_string());
	// }

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

			// why, rust
			let window_clone = window.clone();
			let url_host = url.host_str().unwrap().to_string();
			let passport_copy = passport;
			let app_clone = app_handle.clone();
			let store = store.unwrap();

			let auth_state = app_clone.state::<Mutex<AuthState>>();
			let mut auth_state = tauri::async_runtime::block_on(auth_state.lock());

			// token is base64 encoded, in the format unknown_token:::ws_token:::unknown_token
			let token = url.as_str().split("token=").nth(1).unwrap_or_default();
			if token.is_empty() {
				println!("No token found in URL");
				auth_state.auth_status = AuthStatus::Failed;

				window_clone
					.emit("moodle_auth", &auth_state.auth_status)
					.unwrap();
				window_clone.close().unwrap();
				return false;
			}

			let token = base64::prelude::BASE64_STANDARD.decode(token).unwrap();
			let token = String::from_utf8(token).unwrap();
			let token_parts: Vec<&str> = token.split(":::").collect();
			if token_parts.len() < 2 {
				println!("Invalid token format");
				auth_state.auth_status = AuthStatus::Failed;

				window_clone
					.emit("moodle_auth", &auth_state.auth_status)
					.unwrap();
				window_clone.close().unwrap();
				return false;
			}

			println!("Received wstoken: {}", token_parts[1]);

			store.set(auth_keys::MOODLE_HOST, &*url_host);
			store.set(auth_keys::WS_TOKEN, token_parts[1]);
			store.set(auth_keys::PASSPORT, passport_copy);

			auth_state.auth_status = AuthStatus::Success;
			window_clone
				.emit("moodle_auth", &auth_state.auth_status)
				.unwrap();

			window_clone
				.get_webview_window("main")
				.unwrap()
				.set_focus()
				.unwrap();
			window_clone.close().unwrap();
			return true;
		})
		.build()
		.unwrap();

	Ok(())
}
