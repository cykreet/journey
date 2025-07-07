use std::future::Future;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Emitter};
use tauri_plugin_store::StoreExt;

use crate::store_keys;

#[derive(Serialize, Deserialize, Clone)]
pub enum SyncStatus {
	Success,
	Failed(String),
	Pending,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SyncTask {
	pub id: String,
	pub name: String,
	pub last_sync: u64,
	pub sync_status: SyncStatus,
	pub error: Option<String>,
}

// name
// last_sync
// sync_status
// error

// todo: no idea if this works

pub async fn revalidate_task<F, Fut, T>(
	app: &AppHandle,
	sync_id: &str,
	name: &str,
	task_fn: F,
) -> ()
where
	F: FnOnce(AppHandle) -> Fut + Send + 'static,
	Fut: Future<Output = Result<T, String>> + Send + 'static,
	T: Send + 'static,
{
	let sync_store = app.store(store_keys::SYNC).unwrap();
	let sync_entry = sync_store.get(sync_id);
	let now = std::time::SystemTime::now()
		.duration_since(std::time::SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_secs();
	if sync_entry.is_some() {
		let last_sync: u64 = sync_entry.unwrap().as_u64().unwrap();
		// 3 minutes
		if now - last_sync < 60 * 3 {
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

	let result = task_fn(app.clone()).await;
	let status = match result {
		Ok(_) => SyncStatus::Success,
		Err(error) => SyncStatus::Failed(error),
	};

	sync_store.set(sync_id, json!(now));
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
}
