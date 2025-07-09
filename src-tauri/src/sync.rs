use core::sync;
use std::future::Future;

use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;
use tauri::{AppHandle, Emitter};
use tauri_plugin_store::StoreExt;

use crate::store_keys;

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

// 3 minutes
const SYNC_TIMEOUT: u64 = 60 * 3;

pub async fn revalidate_task<F, Fut, T>(
	app: &AppHandle,
	sync_id: &str,
	name: &str,
	task_fn: F,
) -> ()
where
	F: FnOnce(AppHandle) -> Fut + Send + 'static,
	Fut: Future<Output = Result<T, Box<dyn std::error::Error>>> + Send + 'static,
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

	let result = task_fn(app.clone())
		.await
		.map_err(|error| error.to_string());
	let status = match result {
		Ok(_) => {
			sync_store.set(sync_id, json!(now));
			SyncStatus::Success
		}
		Err(error) => {
			// sync_store.set(sync_id, json!(now + (SYNC_TIMEOUT / 2)));
			SyncStatus::Failed(error)
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
}
