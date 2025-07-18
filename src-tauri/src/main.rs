#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use entity::course::Model as Course;
use entity::course_section::Model as CourseSection;
use entity::course_section_item::Model as CourseSectionItem;
use entity::module_content::Model as ModuleContent;

use specta_typescript::{formatter, BigIntExportBehavior, Typescript};
use tauri::async_runtime::Mutex;
use tauri::{Emitter, Manager};
// use tauri_plugin_store::StoreExt;
use tauri_specta::{collect_commands, Builder};

use self::auth::{get_user_session, open_login_window, AuthState, AuthStatus};
use self::request::course::{get_course, get_user_courses};
use self::sync::SyncTask;

mod auth;
mod database;
mod request;
mod sync;

pub mod store_keys {
	pub const AUTH: &str = "auth";
	pub const SYNC: &str = "sync";
}

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
		.on_window_event(|window, event| match event {
			tauri::WindowEvent::CloseRequested { .. } => {
				let auth_state = window.app_handle().state::<Mutex<AuthState>>();
				let auth_state = tauri::async_runtime::block_on(auth_state.lock());
				if window.label() == "login" && auth_state.status == AuthStatus::Pending {
					// todo: replace with event keys somewhere
					window.emit("login_closed", AuthStatus::Aborted).unwrap();
				}
			}
			_ => {}
		})
		// .plugin(tauri_plugin_updater::Builder::new().build())
		.setup(move |app| {
			builder.mount_events(app);
			let handle = app.handle();

			#[cfg(desktop)]
			handle
				.plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
				.expect("failed to initialise single instance");

			tauri::async_runtime::block_on(async move {
				let database = database::Database::new(&handle)
					.await
					.expect("failed to create database");

				handle.manage(database::DatabaseState(database.connection));
			});

			// todo: set auth state based on existing session
			handle.manage(Mutex::new(AuthState::default()));
			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
