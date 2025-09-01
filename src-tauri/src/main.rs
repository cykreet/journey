#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;

use entity::course::Model as Course;
use entity::course_section::Model as CourseSection;
use entity::module_content::Model as ModuleContent;
use entity::section_module::{Model as CourseSectionItem, SectionModuleType};

use specta_typescript::{BigIntExportBehavior, Typescript};
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Manager, RunEvent, WindowEvent};
use tauri_plugin_fs::FsExt;
use tauri_plugin_store::StoreExt;
use tauri_plugin_updater::UpdaterExt;
use tauri_specta::{Builder, Event, collect_commands, collect_events};

use crate::auth::{AuthState, AuthStatus, MoodleAuthEvent, auth_keys, open_login_window};
use crate::request::course::{
	CourseSectionWithModules, CourseWithSections, SUPPORTED_MODULE_TYPES, SUPPORTED_RESOURCE_TYPES,
	get_content_blobs, get_course, get_module_content, get_user_courses,
};
use crate::sync_task::{ModuleErrorEvent, SyncState};

const MIN_WINDOW_WIDTH: f64 = 300.0;
const MIN_WINDOW_HEIGHT: f64 = 300.0;

mod auth;
mod database;
mod request;
mod sync_task;

pub fn main() {
	let builder = Builder::<tauri::Wry>::new()
		.constant("SUPPORTED_MODULE_TYPES", SUPPORTED_MODULE_TYPES)
		.constant("SUPPORTED_RESOURCE_TYPES", SUPPORTED_RESOURCE_TYPES)
		.commands(collect_commands![
			open_login_window,
			get_user_courses,
			get_course,
			get_module_content,
			get_content_blobs
		])
		.events(collect_events![MoodleAuthEvent, ModuleErrorEvent])
		.typ::<Course>()
		.typ::<CourseSection>()
		.typ::<CourseSectionItem>()
		.typ::<ModuleContent>()
		.typ::<CourseWithSections>()
		.typ::<CourseSectionWithModules>()
		.typ::<AuthStatus>()
		.typ::<SectionModuleType>();

	let ts_exporter = Typescript::new()
		.bigint(BigIntExportBehavior::BigInt)
		.formatter(|path| {
			Command::new("biome")
				.args(["lint", "--write", path.to_str().unwrap()])
				.status()
				.map(|_| ())
		});

	#[cfg(debug_assertions)]
	builder
		.export(ts_exporter, "../src/bindings.ts")
		.expect("failed to export typescript bindings");

	tauri::Builder::default()
		.plugin(tauri_plugin_window_state::Builder::new().build())
		.plugin(tauri_plugin_fs::init())
		.plugin(tauri_plugin_shell::init())
		.invoke_handler(builder.invoke_handler())
		.plugin(tauri_plugin_store::Builder::new().build())
		.plugin(tauri_plugin_http::init())
		.plugin(tauri_plugin_opener::init())
		.plugin(
			tauri_plugin_log::Builder::new()
				.level(log::LevelFilter::Debug)
				.build(),
		)
		.plugin(tauri_plugin_updater::Builder::new().build())
		.setup(move |app| {
			builder.mount_events(app);
			let app_handle = app.handle().clone();
			let scope = app.fs_scope();
			scope
				.allow_directory(app.path().app_local_data_dir().unwrap(), true)
				.ok();

			#[cfg(desktop)]
			app_handle
				.plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
				.expect("failed to initialise single instance");

			app_handle.manage(Mutex::new(SyncState::default()));
			app_handle.manage(Mutex::new(AuthState::default()));

			// #[cfg(debug_assertions)]
			// console_subscriber::init();

			let window_url = match app_handle.store("store.json") {
				Ok(store) => {
					if store.has(auth_keys::USER_ID) && store.has(auth_keys::WS_TOKEN) {
						tauri::WebviewUrl::App("/home".into())
					} else {
						tauri::WebviewUrl::App("/".into())
					}
				}
				Err(e) => {
					log::error!("Failed to open store, forced to setup: {}", e);
					tauri::WebviewUrl::App("/".into())
				}
			};

			let mut win_builder = tauri::WebviewWindowBuilder::new(&app_handle, "main", window_url)
				.title("journey")
				.resizable(true)
				.fullscreen(false)
				.center()
				.inner_size(1200.0, 600.0)
				.visible(false) // prevent flashing when restoring state, will be visible after
				.min_inner_size(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT);

			#[cfg(target_os = "macos")]
			{
				use tauri::TitleBarStyle;
				win_builder = win_builder
					.hidden_title(true)
					.title_bar_style(TitleBarStyle::Overlay);
			}
			#[cfg(not(target_os = "macos"))]
			{
				win_builder = win_builder.decorations(false);
			}

			win_builder.build().unwrap();
			tauri::async_runtime::block_on(async move {
				let database = database::Database::new(&app_handle)
					.await
					.expect("failed to connect to database");
				app_handle.manage(database::DatabaseState(database.connection));
			});

			Ok(())
		})
		.build(tauri::generate_context!())
		.expect("error while running tauri application")
		.run(|app_handle, event| match event {
			RunEvent::WindowEvent {
				event: WindowEvent::Focused(true),
				..
			} => {
				#[cfg(not(debug_assertions))]
				{
					let handle = app_handle.clone();
					// todo: maybe just have an auto update config option
					tauri::async_runtime::spawn(async move {
						update(handle).await.ok();
					});
				}
			}
			RunEvent::WindowEvent {
				event: WindowEvent::CloseRequested { .. },
				label,
				..
			} => {
				if label == "login" {
					let window = app_handle.get_webview_window(&label).unwrap();
					let auth_state = app_handle.state::<Mutex<AuthState>>();
					let mut auth_state = tauri::async_runtime::block_on(auth_state.lock());
					if auth_state.auth_status == AuthStatus::Pending {
						auth_state.auth_status = AuthStatus::Aborted;
						MoodleAuthEvent(auth_state.auth_status.clone())
							.emit(&window)
							.unwrap();
					}
				}
			}
			_ => {}
		})
}

async fn update(app: AppHandle) -> tauri_plugin_updater::Result<()> {
	let update = app.updater()?.check().await?;
	if update.is_none() {
		log::info!("No updates available");
		return Ok(());
	}

	let update = update.unwrap();
	let mut downloaded = 0;
	update
		.download_and_install(
			|chunk_length, content_length| {
				downloaded += chunk_length;
				log::info!("Downloaded {downloaded} from {content_length:?}");
			},
			|| {
				log::info!("Update downloaded, restarting application");
			},
		)
		.await?;

	log::info!("Update downloaded, restarting application");
	app.restart()
}
