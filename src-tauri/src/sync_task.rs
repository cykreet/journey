use std::{collections::HashMap, future::Future, pin::Pin};

use serde_json::json;
use tauri::{AppHandle, Manager, async_runtime::Mutex};

use crate::database::DatabaseState;

#[derive(Default)]
pub struct SyncState {
	pub tasks: HashMap<String, serde_json::Value>,
}

// 3 minutes
const SYNC_TIMEOUT: u64 = 60 * 3;

// todo: this could probably just be a macro
pub struct SyncTask<T> {
	pub app_handle: AppHandle,
	pub sync_id: String,
	pub return_fn: Option<
		Box<
			dyn FnOnce(
					tauri::State<'_, DatabaseState>,
				) -> Pin<Box<dyn Future<Output = Result<T, Box<dyn std::error::Error>>> + Send>>
				+ Send
				+ 'static,
		>,
	>,
}

impl<T> SyncTask<T>
where
	T: Send + 'static,
{
	pub fn new(app: AppHandle, sync_id: String) -> Self {
		Self {
			app_handle: app,
			sync_id: format!("sync_task_{}", sync_id),
			return_fn: None,
		}
	}

	pub fn return_state<F>(mut self, return_fn: F) -> Self
	where
		F: FnOnce(
				tauri::State<'_, DatabaseState>,
			) -> Pin<Box<dyn Future<Output = Result<T, Box<dyn std::error::Error>>> + Send>>
			+ Send
			+ 'static,
	{
		self.return_fn = Some(Box::new(return_fn));
		self
	}

	pub async fn sync_state<F>(mut self, task_fn: F) -> Result<T, Box<dyn std::error::Error>>
	where
		F: FnOnce(
				AppHandle,
			) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>>
			+ Send
			+ 'static,
	{
		let sync_state = self.app_handle.state::<Mutex<SyncState>>();
		let mut sync_state = sync_state.lock().await;
		let db_state = self.app_handle.state::<DatabaseState>();
		let sync_entry = sync_state.tasks.get(&self.sync_id);
		let now = std::time::SystemTime::now()
			.duration_since(std::time::SystemTime::UNIX_EPOCH)
			.unwrap()
			.as_secs();
		if sync_entry.is_some() {
			let last_sync: u64 = sync_entry.unwrap().as_u64().unwrap();
			if now - last_sync < SYNC_TIMEOUT {
				return self
					.return_fn
					.take()
					.map(|f| f(db_state))
					.unwrap_or_else(|| panic!("Sync task {} did not return a function", self.sync_id))
					.await;
			}
		}

		match task_fn(self.app_handle.clone()).await {
			Ok(_) => {
				sync_state
					.tasks
					.insert(self.sync_id.to_string(), json!(now));
			}
			Err(e) => {
				log::error!("Error in sync task {}: {}", self.sync_id, e);
				return Err(e);
			}
		};

		self
			.return_fn
			.take()
			.map(|f| f(db_state))
			.unwrap_or_else(|| panic!("Sync task {} did not return a function", self.sync_id))
			.await
	}
}
