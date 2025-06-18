use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{
	webview::{Cookie, PageLoadEvent},
	AppHandle, Emitter, Manager, Url, WebviewWindowBuilder,
};
use tauri_plugin_http::reqwest;
use tauri_plugin_store::StoreExt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum AuthState {
	Failed,
	Success,
	Aborted,
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
	let auth_store = app.store("auth").unwrap();
	let session_cookie = auth_store.get(auth_keys::MOODLE_SESSION).unwrap();
	Ok(session_cookie.as_str().unwrap().to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn open_login_window(app: AppHandle, domain: &str) -> Result<(), String> {
	let window_url = Url::parse(domain).unwrap();
	let auth_store = app.store("auth").unwrap();
	auth_store.delete(auth_keys::INITIAL_SESSION);

	let app_handle = app.clone();
	let window_label = "login";
	WebviewWindowBuilder::new(
		&app,
		window_label,
		tauri::WebviewUrl::External(window_url.clone()),
	)
	.on_page_load(move |window, payload| match payload.event() {
		PageLoadEvent::Started => (),
		PageLoadEvent::Finished => {
			let store = window.store("auth");
			if store.is_err() {
				println!("could not open store");
				return window.close().unwrap();
			}

			let url = payload.url();
			let host_cookies = window.cookies_for_url(url.clone()).unwrap();
			let session_cookie = get_session_cookie(&host_cookies);
			if session_cookie.is_none() && window_url.host() == url.host() {
				println!("not a valid moodle instance");
				return window.close().unwrap();
			}
		}
	})
	.on_navigation(move |url| {
		let window = app_handle.get_webview_window(window_label).unwrap();
		let store = window.store("auth");
		if store.is_err() {
			println!("could not open store");
			window.close().unwrap();
			return true;
		}

		let auth_store = store.unwrap();
		let host_cookies = window.cookies_for_url(url.clone()).unwrap();
		let session_cookie = get_session_cookie(&host_cookies);
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

				// todo: don't think this is too great, we'll need to redo the request
				// when the session key expires in other places
				let cloned_auth_store = auth_store.clone();
				let cloned_url = url.clone();
				let cloned_session_value = session_value.clone();
				tauri::async_runtime::spawn(async move {
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
					cloned_auth_store.set(auth_keys::SESSION_KEY, session_key);
				});

				auth_store.set(auth_keys::MOODLE_SESSION, session_value);
				auth_store.set(auth_keys::MOODLE_HOST, url.host().unwrap().to_string());
				window.emit("login_close", AuthState::Success).unwrap();
				window.close().unwrap();
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
