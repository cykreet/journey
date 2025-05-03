use tauri::{
	webview::{Cookie, PageLoadEvent},
	AppHandle, Manager, Url, WebviewWindowBuilder,
};
use tauri_plugin_store::StoreExt;

pub mod auth_keys {
	pub const INITIAL_SESSION: &str = "initial_session";
	pub const MOODLE_SESSION: &str = "moodle_session";
	pub const MOODLE_HOST: &str = "moodle_host";
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
				if initial_session == session_value {
					println!("session cookie has not changed, probably not logged in");
					return true;
				}

				println!("storing session {}", session_value);
				auth_store.set(auth_keys::MOODLE_SESSION, session_value);
				auth_store.set(auth_keys::MOODLE_HOST, url.host().unwrap().to_string());
				window.close().unwrap();
			}
			None => {
				println!(
					"storing initial session {}",
					session_cookie.unwrap().value()
				);
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
	return cookies
		.iter()
		.find(|cookie| cookie.name() == "MoodleSession");
}
