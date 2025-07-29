#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;
use std::{hint, thread};

use entity::course::Model as Course;
use entity::course_section::Model as CourseSection;
use entity::course_section_item::Model as CourseSectionItem;
use entity::module_content::Model as ModuleContent;

use specta_typescript::{formatter, BigIntExportBehavior, Typescript};
use tauri::async_runtime::Mutex;
use tauri::{Emitter, Manager, Url};
use tauri_plugin_store::StoreExt;
use tauri_specta::{collect_commands, Builder};

use crate::auth::auth_keys;
use crate::request::course::{CourseSectionWithItems, CourseWithSections};
use crate::request::session::{self, SessionStatus};
use crate::sync::SyncState;

use self::auth::{get_user_session, open_login_window, AuthState, AuthStatus};
use self::request::course::{get_course, get_user_courses};
use self::sync::SyncTask;

mod auth;
mod database;
mod request;
mod sync;

const SESSION_POLL_INTERVAL: Duration = Duration::from_secs(5 * 60);
const SESSION_TOUCH_THRESHOLD: Duration = Duration::from_secs(60 * 60);

pub fn main() {
	let builder = Builder::<tauri::Wry>::new()
		.commands(collect_commands![
			open_login_window,
			get_user_courses,
			get_course,
			get_user_session,
		])
		.typ::<Course>()
		.typ::<CourseSection>()
		.typ::<CourseSectionItem>()
		.typ::<ModuleContent>()
		.typ::<CourseWithSections>()
		.typ::<CourseSectionWithItems>()
		.typ::<SyncTask>()
		.typ::<AuthStatus>();

	let ts_exporter = Typescript::new()
		.bigint(BigIntExportBehavior::BigInt)
		.formatter(formatter::biome);

	#[cfg(debug_assertions)]
	builder
		.export(ts_exporter, "../src/bindings.ts")
		.expect("failed to export typescript bindings");

	tauri::Builder::default()
		.plugin(tauri_plugin_log::Builder::new().build())
		.invoke_handler(builder.invoke_handler())
		.plugin(tauri_plugin_store::Builder::new().build())
		.plugin(tauri_plugin_http::init())
		.plugin(tauri_plugin_opener::init())
		.plugin(tauri_plugin_log::Builder::new().build())
		// .plugin(tauri_plugin_updater::Builder::new().build())
		.on_window_event(|window, event| match event {
			tauri::WindowEvent::CloseRequested { .. } => {
				let auth_state = window.app_handle().state::<Mutex<AuthState>>();
				let auth_state = tauri::async_runtime::block_on(auth_state.lock());
				if window.label() == "login" && auth_state.auth_status == AuthStatus::Pending {
					// todo: replace with event keys somewhere
					window.emit("login_closed", AuthStatus::Aborted).unwrap();
				}
			}
			_ => {}
		})
		.setup(move |app| {
			builder.mount_events(app);
			let handle = app.handle();

			#[cfg(desktop)]
			handle
				.plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
				.expect("failed to initialise single instance");

			// #[cfg(debug_assertions)]
			// console_subscriber::init();

			tauri::async_runtime::block_on(async {
				let database = database::Database::new(&handle)
					.await
					.expect("failed to connect to database");
				handle.manage(database::DatabaseState(database.connection));
				handle.manage(Mutex::new(SyncState::default()));
			});

			// spawn a task to poll session status, emits session_change events
			// relevant to whether the session is valid, expired, or invalid
			// since moodle supports oauth and other login methods, we can't
			// realistically support re-authentication, so we just notify the user
			// when the session is no longer valid after we've tried revalidating
			let app_handle = handle.clone();
			tauri::async_runtime::spawn(async move {
				loop {
					let start = std::time::Instant::now();
					let store = app_handle.store("store.json").unwrap();
					let host = store.get(auth_keys::MOODLE_HOST);
					if host.is_none() {
						app_handle
							.emit("session_change", SessionStatus::Invalid)
							.unwrap();

						hint::spin_loop();
						thread::sleep(SESSION_POLL_INTERVAL - start.elapsed());
						continue;
					}

					let host = host.unwrap().to_string();
					let session_cookie = store.get(auth_keys::MOODLE_SESSION).or({
						let window = app_handle.get_webview_window("main").unwrap();
						let url = Url::parse(&host).unwrap();
						let cookies = window.cookies_for_url(url).unwrap();
						session::find_session_cookie(&cookies).map(|cookie| serde_json::json!(cookie.value()))
					});

					if session_cookie.is_none() {
						app_handle
							.emit("session_change", SessionStatus::Invalid)
							.unwrap();

						hint::spin_loop();
						thread::sleep(SESSION_POLL_INTERVAL - start.elapsed());
						continue;
					}

					let session_cookie = session_cookie.unwrap().to_string();
					let session_key = store.get(auth_keys::SESSION_KEY);
					match session_key {
						Some(session_key) => {
							let remaining_time = session::get_remaining_time(
								Url::parse(&host).unwrap(),
								&session_cookie,
								&session_key.to_string(),
							)
							.await;
							match remaining_time {
								Ok(remaining_time) => {
									if remaining_time == 0 {
										// session cookie is still valid, but session key might have expired,
										// need to refetch host to get a new one
										session::fetch_session_key(
											Url::parse(&host).unwrap(),
											session_cookie.to_string(),
										)
										.await;
									} else if remaining_time < SESSION_TOUCH_THRESHOLD.as_secs() {
										// session key is still alive, but we can touch it to extend it
										let _ = session::touch_session(
											Url::parse(&host).unwrap(),
											&session_cookie,
											&session_key.to_string(),
										)
										.await;
									}

									app_handle
										.emit("session_change", SessionStatus::Valid)
										.unwrap();
								}
								Err(_) => {
									// session cookie is no longer valid, and we'll need to relogin
									app_handle
										.emit("session_change", SessionStatus::Expired)
										.unwrap();
								}
							}
						}
						None => {
							// session key is not stored for whatever reason, might be able to fetch
							// a new one if the session cookie is still valid/stored
							let session_key =
								session::fetch_session_key(Url::parse(&host).unwrap(), session_cookie.to_string())
									.await;

							app_handle
								.emit("session_change", SessionStatus::Valid)
								.unwrap();
						}
					}

					hint::spin_loop();
					thread::sleep(SESSION_POLL_INTERVAL - start.elapsed());
				}
			});

			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
