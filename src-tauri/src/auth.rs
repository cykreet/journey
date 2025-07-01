use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{
	async_runtime::Mutex, webview::Cookie, AppHandle, Emitter, Manager, Url, WebviewWindowBuilder,
};
use tauri_plugin_http::reqwest;
use tauri_plugin_store::StoreExt;

use crate::store_keys;

#[derive(Default)]
pub struct AuthState {
	pub status: AuthStatus,
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

pub mod auth_keys {
	pub const INITIAL_SESSION: &str = "initial_session";
	pub const MOODLE_SESSION: &str = "moodle_session";
	pub const MOODLE_HOST: &str = "moodle_host";
	pub const SESSION_KEY: &str = "session_key";
}

#[tauri::command]
#[specta::specta]
pub async fn get_user_session(app: AppHandle) -> Result<String, String> {
	let auth_store = app.store(store_keys::AUTH).unwrap();
	let session_cookie = auth_store.get(auth_keys::MOODLE_SESSION).unwrap();
	Ok(session_cookie.as_str().unwrap().to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn open_login_window(app: AppHandle, domain: &str) -> Result<(), String> {
	if domain.is_empty() {
		return Err("invalid domain".to_string());
	}

	let window_url = Url::parse(domain).unwrap();
	let auth_store = app.store(store_keys::AUTH).unwrap();
	auth_store.delete(auth_keys::INITIAL_SESSION);

	let auth_state = app.state::<Mutex<AuthState>>();
	let mut auth_state = auth_state.lock().await;
	auth_state.status = AuthStatus::Pending;

	let validate_response = reqwest::Client::new()
		.get(window_url.clone())
		.send()
		.await
		.unwrap();

	let mut validate_cookies = validate_response.cookies();
	let has_moodle_session = validate_cookies.any(|cookie| cookie.name() == "MoodleSession");
	// the session cookie should be present on all requests relevant to a (possibly unmodified)
	// moodle instance, if it's not present and we're on the requested url, it's probably not a
	// valid instance.
	if validate_response.status().is_success() == false || has_moodle_session == false {
		return Err("invalid instance".to_string());
	}

	let app_handle = app.clone();
	let window_label = "login";
	WebviewWindowBuilder::new(
		&app,
		window_label,
		tauri::WebviewUrl::External(window_url.clone()),
	)
	.on_navigation(move |url| {
		let window = app_handle.get_webview_window(window_label).unwrap();
		let store = window.store(store_keys::AUTH);
		if store.is_err() {
			println!("could not open store");
			window.close().unwrap();
			return false;
		}

		let auth_store = store.unwrap();
		let host_cookies = window.cookies_for_url(url.clone()).unwrap();
		let session_cookie = get_session_cookie(&host_cookies);
		// if the session cookie isn't present at this point and we're not on the request url, we're probably
		// just navigating (oauth flows, etc.) and don't necessarily have to panic.
		if session_cookie.is_none() {
			return true;
		}

		match auth_store.get(auth_keys::INITIAL_SESSION) {
			Some(initial_session) => {
				let session_value = session_cookie.unwrap().value().to_string();
				// session cookie has not changed, probably not logged in
				if initial_session == session_value {
					return true;
				}

				let cloned_url = url.clone();
				let owned_app_handle = app_handle.clone();
				let cloned_session_value = session_value.clone();
				auth_store.set(auth_keys::MOODLE_SESSION, session_value);
				auth_store.set(auth_keys::MOODLE_HOST, url.host().unwrap().to_string());

				tauri::async_runtime::spawn(async move {
					// todo: we'll need to redo the request
					// when the session key expires in other places
					let response = reqwest::Client::new()
						.get(cloned_url)
						.header("Cookie", format!("MoodleSession={}", cloned_session_value))
						.send()
						.await
						.unwrap();

					let body = response.text().await.unwrap();
					let session_key_start = body.find(r#""sesskey":""#).unwrap() + 11;
					let session_key_end = body[session_key_start..].find('"').unwrap() + session_key_start;
					let session_key = &body[session_key_start..session_key_end];

					let auth_state = owned_app_handle.state::<Mutex<AuthState>>();
					let mut auth_state = auth_state.lock().await;
					auth_state.status = if session_key.is_empty() {
						AuthStatus::Failed
					} else {
						AuthStatus::Success
					};

					auth_store.set(auth_keys::SESSION_KEY, session_key);
					window.emit("login_closed", &auth_state.status).unwrap();
					window.close().unwrap();
				});
			}
			None => {
				auth_store.set(auth_keys::INITIAL_SESSION, session_cookie.unwrap().value());
			}
		}

		return true;
	})
	.build()
	.unwrap();

	Ok(())
}

fn get_session_cookie<'a>(cookies: &'a Vec<Cookie<'static>>) -> Option<&'a Cookie<'a>> {
	cookies
		.iter()
		.find(|cookie| cookie.name() == "MoodleSession")
}
