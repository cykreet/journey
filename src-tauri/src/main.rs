#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use entity::course::Model as Course;
use entity::course_section::Model as CourseSection;
use entity::module_content::Model as ModuleContent;
use entity::section_module::Model as CourseSectionItem;

use specta_typescript::{formatter, BigIntExportBehavior, Typescript};
use tauri::async_runtime::Mutex;
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tauri_specta::{collect_commands, collect_events, Builder, Event};

use crate::auth::{auth_keys, open_login_window, AuthState, AuthStatus, MoodleAuthEvent};
use crate::request::course::{
	get_course, get_module_content, get_user_courses, CourseSectionWithItems, CourseWithSections,
	SUPPORTED_MODULE_TYPES,
};
use crate::sync_task::SyncState;

mod auth;
mod database;
mod request;
mod sync_task;

pub fn main() {
	let builder = Builder::<tauri::Wry>::new()
		.constant("SUPPORTED_MODULE_TYPES", SUPPORTED_MODULE_TYPES)
		.commands(collect_commands![
			open_login_window,
			get_user_courses,
			get_course,
			get_module_content
		])
		.events(collect_events![MoodleAuthEvent])
		.typ::<Course>()
		.typ::<CourseSection>()
		.typ::<CourseSectionItem>()
		.typ::<ModuleContent>()
		.typ::<CourseWithSections>()
		.typ::<CourseSectionWithItems>()
		.typ::<AuthStatus>();

	let ts_exporter = Typescript::new()
		.bigint(BigIntExportBehavior::BigInt)
		.formatter(formatter::biome);

	#[cfg(debug_assertions)]
	builder
		.export(ts_exporter, "../src/bindings.ts")
		.expect("failed to export typescript bindings");

	tauri::Builder::default()
		.invoke_handler(builder.invoke_handler())
		.plugin(tauri_plugin_store::Builder::new().build())
		.plugin(tauri_plugin_http::init())
		.plugin(tauri_plugin_opener::init())
		.plugin(
			tauri_plugin_log::Builder::new()
				.level(log::LevelFilter::Debug)
				.build(),
		)
		// .plugin(tauri_plugin_updater::Builder::new().build())
		.on_window_event(|window, event| match event {
			tauri::WindowEvent::CloseRequested { .. } => {
				let auth_state = window.app_handle().state::<Mutex<AuthState>>();
				let auth_state = tauri::async_runtime::block_on(auth_state.lock());
				if window.label() == "login" && auth_state.auth_status == AuthStatus::Pending {
					MoodleAuthEvent(AuthStatus::Aborted).emit(window).unwrap();
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
				handle.manage(Mutex::new(AuthState::default()));

				// initial setup checks should not actually validate the session,
				// just check if we've been authenticated before.
				let store = handle.store("store.json");
				if store.is_err() {
					log::error!("Failed to open store.json, forced to setup");
					return;
				}

				let store = store.unwrap();
				if let Some(window) = handle.get_webview_window("main") {
					if store.has(auth_keys::USER_ID) == false || store.has(auth_keys::WS_TOKEN) == false {
						return;
					}

					window.eval("window.location.replace('/home');").unwrap();
				}
			});

			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
