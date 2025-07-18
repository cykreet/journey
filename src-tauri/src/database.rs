use std::{
	env::{self, set_var},
	fs::{create_dir_all, remove_file},
	time::Duration,
};

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, DatabaseConnection};
use tauri::{AppHandle, Manager};

pub struct Database {
	pub connection: DatabaseConnection,
}

#[allow(dead_code)]
pub struct DatabaseState(pub DatabaseConnection);

impl Database {
	pub async fn new(app_handle: &AppHandle) -> Result<Self, String> {
		let app_dir = app_handle
			.path()
			.app_data_dir()
			.expect("failed to get app data dir");

		// CHANGEEGE
		create_dir_all(&app_dir).map_err(|e| format!("Failed to create app data directory: {}", e))?;
		let db_path = app_dir.join("journey.db");
		set_var(
			"DATABASE_URL",
			format!("sqlite://{}?mode=rwc", db_path.display()),
		);

		// true for dev command or build --debug, false otherwise
		// https://tauri.app/reference/environment-variables/
		if env::var("TAURI_ENV_DEBUG").is_ok() {
			remove_file(&db_path).ok();
		}

		let mut options = ConnectOptions::new(&env::var("DATABASE_URL").unwrap());
		options
			.connect_timeout(Duration::from_secs(8))
			.acquire_timeout(Duration::from_secs(8))
			.idle_timeout(Duration::from_secs(8))
			.max_lifetime(Duration::from_secs(8))
			.sqlx_logging(true);

		let db = sea_orm::Database::connect(options)
			.await
			.map_err(|e| format!("Failed to connect to database: {}", e))?;

		Migrator::up(&db, None)
			.await
			.map_err(|e| format!("Failed to run migrations: {}", e))?;
		Ok(Self { connection: db })
	}
}
