#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use specta_typescript::{formatter, BigIntExportBehavior, Typescript};
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

use auth::open_login_window;
use entities::{ContentType, Course, CourseItem, CourseItemContent};
use service_request::get_user_courses;

mod auth;
mod database;
mod entities;
mod service_request;
mod sql_query;

pub fn main() {
	let builder = Builder::<tauri::Wry>::new()
		.commands(collect_commands![open_login_window, get_user_courses])
		.typ::<Course>()
		.typ::<ContentType>()
		.typ::<CourseItem>()
		.typ::<CourseItemContent>();

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

			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
