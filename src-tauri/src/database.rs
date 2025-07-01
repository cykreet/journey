use std::{
	env::{self, set_var},
	fs::{create_dir_all, remove_file},
};

use sqlx::{
	sqlite::{SqliteConnectOptions, SqliteJournalMode},
	Pool, Sqlite, SqlitePool,
};
use tauri::{AppHandle, Manager};

pub struct Database {
	pub pool: Pool<Sqlite>,
}

#[allow(dead_code)]
pub struct DatabaseState(pub Pool<Sqlite>);

impl Database {
	pub async fn new(app_handle: &AppHandle) -> Result<Self, sqlx::Error> {
		let app_dir = app_handle
			.path()
			.app_data_dir()
			.expect("failed to get app data dir");

		create_dir_all(&app_dir)?;
		let db_path = app_dir.join("journey.db");
		set_var("DATABASE_URL", format!("sqlite://{}", db_path.display()));

		let connection_options = SqliteConnectOptions::new()
			.filename(&db_path)
			.create_if_missing(true)
			.journal_mode(SqliteJournalMode::Wal);

		let pool = SqlitePool::connect_with(connection_options).await?;
		sqlx::migrate!("./migrations").run(&pool).await?;
		Ok(Self { pool })
	}
}
