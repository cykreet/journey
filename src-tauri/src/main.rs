#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use specta_typescript::{formatter, BigIntExportBehavior, Typescript};
use tauri::{Emitter, Manager};
// use tauri_plugin_store::StoreExt;
use tauri_specta::{collect_commands, Builder};

use auth::{get_user_session, open_login_window, AuthState, AuthStatus};
use entities::{ContentType, Course, CourseItem, CourseItemContent};
use request::courses::get_user_courses;

mod auth;
mod database;
mod entities;
mod request;
mod sql_query;

pub fn main() {
	let builder = Builder::<tauri::Wry>::new()
		.commands(collect_commands![
			open_login_window,
			get_user_courses,
			get_user_session
		])
		.typ::<Course>()
		.typ::<ContentType>()
		.typ::<CourseItem>()
		.typ::<CourseItemContent>()
		.typ::<AuthStatus>();

	let ts_exporter = Typescript::new()
		.bigint(BigIntExportBehavior::BigInt)
		.formatter(formatter::biome);

	// todo: will this break in release builds?
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
				let auth_state = auth_state.lock().unwrap();
				if window.label() == "login" && auth_state.status == AuthStatus::Pending {
					// todo: replace with event keys somewhere
					window.emit("login_closed", AuthStatus::Aborted).unwrap();
					// set
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
				.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {}))
				.expect("failed to initialise single instance");

			tauri::async_runtime::block_on(async move {
				let database = database::Database::new(&handle)
					.await
					.expect("failed to create database");

				handle.manage(database::DatabaseState(database.pool));
			});

			// todo: set auth state based on existing session
			handle.manage(Mutex::new(AuthState::default()));
			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
