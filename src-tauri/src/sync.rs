use std::{collections::HashMap, future::Future};

use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;
use tauri::{async_runtime::Mutex, AppHandle, Emitter, Manager};

#[derive(Serialize, Deserialize, Clone, Type, Debug)]
pub enum SyncStatus {
	Success,
	Failed(String),
	Pending,
}

#[derive(Serialize, Deserialize, Clone, Type)]
pub struct SyncTask {
	pub id: String,
	pub name: String,
	pub last_sync: u64,
	pub sync_status: SyncStatus,
	pub error: Option<String>,
}

#[derive(Default)]
pub struct SyncState {
	pub tasks: HashMap<String, serde_json::Value>,
}

// 3 minutes
const SYNC_TIMEOUT: u64 = 60 * 3;

pub async fn revalidate_task<F, Fut, T>(
	app: AppHandle,
	sync_id: String,
	name: String,
	task_fn: F,
) -> ()
where
	F: FnOnce(AppHandle) -> Fut + Send + 'static,
	Fut: Future<Output = Result<T, Box<dyn std::error::Error>>> + Send + 'static,
	T: Send + 'static,
{
	tauri::async_runtime::spawn(async move {
		let sync_state = app.state::<Mutex<SyncState>>();
		let mut sync_state = sync_state.lock().await;
		let sync_entry = sync_state.tasks.get(&sync_id);
		let now = std::time::SystemTime::now()
			.duration_since(std::time::SystemTime::UNIX_EPOCH)
			.unwrap()
			.as_secs();
		if sync_entry.is_some() {
			let last_sync: u64 = sync_entry.unwrap().as_u64().unwrap();
			if now - last_sync < SYNC_TIMEOUT {
				return;
			}
		}

		app
			.emit(
				"sync_task",
				SyncTask {
					id: sync_id.to_string(),
					name: name.to_string(),
					last_sync: now,
					sync_status: SyncStatus::Pending,
					error: None,
				},
			)
			.unwrap();

		// todo: add returned data to event payload
		let result = task_fn(app.clone()).await.map_err(|e| e.to_string());
		let status = match result {
			Ok(_) => {
				sync_state.tasks.insert(sync_id.to_string(), json!(now));
				SyncStatus::Success
			}
			Err(e) => {
				// sync_store.set(sync_id, json!(now + (SYNC_TIMEOUT / 2)));
				SyncStatus::Failed(e)
			}
		};

		println!("Sync task {} finished with status {:?}", sync_id, status);

		app
			.emit(
				"sync_task",
				SyncTask {
					id: sync_id.to_string(),
					name: name.to_string(),
					last_sync: now,
					sync_status: status,
					error: None,
				},
			)
			.unwrap();
	});
}
