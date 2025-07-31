#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use entity::course::Model as Course;
use entity::course_section::Model as CourseSection;
use entity::course_section_item::Model as CourseSectionItem;
use entity::module_content::Model as ModuleContent;

use specta_typescript::{formatter, BigIntExportBehavior, Typescript};
use tauri::async_runtime::Mutex;
use tauri::{Emitter, Manager};
use tauri_specta::{collect_commands, Builder};

use crate::auth::{open_login_window, AuthState, AuthStatus};
use crate::request::course::{
	get_course, get_user_courses, CourseSectionWithItems, CourseWithSections,
};
use crate::sync::{SyncState, SyncTask};

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
					window.emit("moodle_auth", AuthStatus::Aborted).unwrap();
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
			});
			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
