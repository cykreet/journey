use std::{ops::Not, vec};

use entity::section_module::SectionModuleType;
use sea_orm::{
	sea_query, ActiveValue, ColumnTrait, Condition, EntityTrait, QueryFilter, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::types::chrono::Utc;
use tauri::{AppHandle, Manager};
use tauri_plugin_http::reqwest;
use tauri_plugin_store::StoreExt;

use crate::{
	auth::auth_keys,
	database::DatabaseState,
	request::rest::{self, RestCourse, RestCourseSection},
	sync_task::SyncTask,
};

#[derive(Serialize, Deserialize, Type)]
pub struct CourseWithSections {
	pub course: entity::course::Model,
	pub sections: Vec<CourseSectionWithItems>,
}

#[derive(Serialize, Deserialize, Type)]
pub struct CourseSectionWithItems {
	pub section: entity::course_section::Model,
	pub items: Vec<entity::section_module::Model>,
}

// we filter for these when revalidating and fetching course data
pub const SUPPORTED_MODULE_TYPES: [SectionModuleType; 4] = [
	SectionModuleType::Page,
	SectionModuleType::Book,
	SectionModuleType::Resource,
	SectionModuleType::Url,
];

// supported mime types relevant to actual module content and not embedded content like images
pub const SUPPORTED_MIME_TYPES: [&str; 2] = ["application/pdf", "text/html"];

#[tauri::command]
#[specta::specta]
pub async fn get_course(app: AppHandle, course_id: i32) -> Result<CourseWithSections, String> {
	SyncTask::new(app, format!("get_course_{}", course_id))
		.return_state(move |state| {
			let db = state.0.clone();
			Box::pin(async move {
				let (course, sections) = entity::Course::find_by_id(course_id)
					.find_with_related(entity::CourseSection)
					.all(&db)
					.await
					.map_err(|e| e.to_string())?
					.into_iter()
					.next()
					.ok_or_else(|| format!("Course with id {} not found", course_id))?;

				let mut sections_with_items = vec![];
				for section in sections {
					let items = entity::SectionModule::find()
						.filter(
							Condition::all()
								.add(entity::section_module::Column::SectionId.eq(section.id))
								.add(entity::section_module::Column::ModuleType.is_in(SUPPORTED_MODULE_TYPES)),
						)
						.all(&db)
						.await
						.map_err(|e| e.to_string())?;

					if items.is_empty().not() {
						sections_with_items.push((section, items));
					}
				}

				Ok(CourseWithSections {
					course: course,
					sections: sections_with_items
						.into_iter()
						.map(|(section, items)| CourseSectionWithItems { section, items })
						.collect(),
				})
			})
		})
		.sync_state(move |app_handle| {
			Box::pin(async move {
				let store = app_handle.store("store.json").unwrap();
				let client = reqwest::Client::new();
				let host = store.get(auth_keys::MOODLE_HOST).unwrap();
				let ws_token = store.get(auth_keys::WS_TOKEN).unwrap();
				let request = rest::get_course_sections_request(
					course_id,
					&client,
					host.as_str().unwrap(),
					ws_token.as_str().unwrap(),
				)
				.map_err(|e| e.to_string())?;

				let response = client.execute(request).await.unwrap();
				if response.status().is_success().not() {
					return Err(format!("Failed to fetch course sections with id: {course_id}").into());
				}

				let body = response.text().await.unwrap();
				if body.contains("errorcode") {
					return Err(format!("Could not get course: {}", body).into());
				}

				let sections_data: Vec<RestCourseSection> =
					serde_json::from_str(&body).map_err(|e| e.to_string())?;
				let state = app_handle.state::<DatabaseState>();
				let db = &state.0;
				let txn = db.begin().await.map_err(|e| e.to_string())?;

				for section in sections_data {
					let section_entity = entity::course_section::ActiveModel {
						id: ActiveValue::Set(section.id),
						name: ActiveValue::Set(section.name.clone()),
						course_id: ActiveValue::Set(course_id),
					};

					entity::CourseSection::insert(section_entity)
						.on_conflict(
							sea_query::OnConflict::column(entity::course_section::Column::Id)
								.update_columns([
									entity::course_section::Column::Name,
									entity::course_section::Column::CourseId,
								])
								.to_owned(),
						)
						.exec(&txn)
						.await
						.map_err(|e| e.to_string())?;

					for module in section.modules {
						if !SUPPORTED_MODULE_TYPES.contains(&module.module_type) {
							continue;
						}

						let section_item = entity::section_module::ActiveModel {
							id: ActiveValue::Set(module.id),
							name: ActiveValue::Set(module.name.clone()),
							section_id: ActiveValue::Set(section.id),
							module_type: ActiveValue::Set(module.module_type),
							updated_at: ActiveValue::Set(Utc::now().timestamp()),
						};

						entity::SectionModule::insert(section_item)
							.on_conflict(
								sea_query::OnConflict::column(entity::section_module::Column::Id)
									.update_columns([
										entity::section_module::Column::Name,
										entity::section_module::Column::SectionId,
										entity::section_module::Column::UpdatedAt,
									])
									.to_owned(),
							)
							.exec(&txn)
							.await
							.map_err(|e| e.to_string())?;
					}
				}

				txn.commit().await.map_err(|e| e.to_string())?;
				Ok(())
			})
		})
		.await
		.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_module_content(
	app: AppHandle,
	course_id: i32,
	module_id: i32,
) -> Result<
	(
		entity::section_module::Model,
		Vec<entity::module_content::Model>,
	),
	String,
> {
	SyncTask::new(app, format!("get_module_content_{}", module_id))
		.return_state(move |state| {
			let db = state.0.clone();
			Box::pin(async move {
				let module_with_content = entity::SectionModule::find_by_id(module_id)
					.find_with_related(entity::ModuleContent)
					.all(&db)
					.await
					.map_err(|error| error.to_string())?;

				if module_with_content.is_empty() {
					return Err(format!("Module with id {} not found", module_id).into());
				}

				// todo: if module is resource, get content blob for resource to return
				// and check if supported mime type, etc.

				Ok(module_with_content[0].clone())
			})
		})
		.sync_state(move |app_handle| {
			Box::pin(async move {
				let store = app_handle.store("store.json").unwrap();
				let client = reqwest::Client::new();
				let token = store.get(auth_keys::WS_TOKEN).unwrap();
				let token = token.as_str().unwrap();
				let request = rest::get_sections_with_model_content(
					course_id,
					module_id,
					&client,
					store.get(auth_keys::MOODLE_HOST).unwrap().as_str().unwrap(),
					token,
				)
				.map_err(|e| e.to_string())?;

				let response = client.execute(request).await.unwrap();
				if response.status().is_success().not() {
					return Err(format!("Failed to fetch module content with id: {module_id}").into());
				}

				let body = response.text().await.unwrap();
				if body.contains("errorcode") {
					return Err(format!("Could not get module content: {}", body).into());
				}

				let sections_data: Vec<RestCourseSection> =
					serde_json::from_str(&body).map_err(|e| e.to_string())?;
				let module = sections_data
					.into_iter()
					.flat_map(|section| section.modules)
					.find(|module| module.id == module_id)
					.ok_or_else(|| format!("Module with id {} not found", module_id))?;

				if SUPPORTED_MODULE_TYPES.contains(&module.module_type).not() {
					return Err(format!("Module type {} is not supported", module.module_type).into());
				}

				let module_contents = module.contents.unwrap_or_default();
				let state = app_handle.state::<DatabaseState>();
				let db = &state.0;
				let txn = db.begin().await.map_err(|e| e.to_string())?;

				for (i, content) in module_contents.iter().enumerate() {
					// ids of the content blocks stored in file path as "/id/"
					// media content also uses this to refer to the relevant content block.
					// the "root" content block seems to always have a path of "/", which is usually
					// included if it doesn't contain any other content
					let content_id = if content.file_path == "/" {
						1
					} else {
						content.file_path[1..content.file_path.len() - 1]
							.parse::<i32>()
							.map_err(|e| e.to_string())?
					};

					// "books" have an additional structure content object that contains the hierarchy of the contents,
					// not sure how i wanna handle books, but storing content blocks as they appear in the response is fine for now

					// written content is usually in an index.html file. we generally wanna store text content directly, blobs being
					// stored on the filesystem, with paths stored in the database.
					if content.file_name == "index.html" {
						let file_url = format!("{}?forcedownload=1&token={}", content.file_url, token);
						let content_response = reqwest::get(file_url).await.map_err(|e| e.to_string())?;
						if content_response.status().is_success().not() {
							return Err(format!("Failed to fetch content for content id: {}", content_id).into());
						}

						// todo: remove any scripts and stylesheets that are not needed
						// html content is usually also pretty ugly with empty tags, etc.
						let content_text = content_response.text().await.map_err(|e| e.to_string())?;
						let module_content = entity::module_content::ActiveModel {
							id: ActiveValue::Set(content_id),
							module_id: ActiveValue::Set(module_id),
							content: ActiveValue::Set(content_text),
							rank: ActiveValue::Set(i as i32),
							updated_at: ActiveValue::Set(Utc::now().timestamp()),
						};

						entity::ModuleContent::insert(module_content)
							.on_conflict(
								sea_query::OnConflict::columns([
									entity::module_content::Column::Id,
									entity::module_content::Column::ModuleId,
								])
								.update_columns([
									entity::module_content::Column::Content,
									entity::module_content::Column::Rank,
									entity::module_content::Column::UpdatedAt,
								])
								.to_owned(),
							)
							.exec(&txn)
							.await
							.map_err(|e| e.to_string())?;
					}

					if let Some(mime_type) = &content.mime_type {
						if SUPPORTED_MIME_TYPES.contains(&mime_type.as_str()).not() {
							continue;
						}

						// todo: either check time modified on module or make HEAD request to check "last-modified" header
						// to avoid downloading the same file again if it hasn't changed
						let file_url = format!("{}?forcedownload=1&token={}", content.file_url, token);
						let content_response = reqwest::get(file_url).await.map_err(|e| e.to_string())?;
						if content_response.status().is_success().not() {
							return Err(
								format!(
									"Failed to fetch content blob for content id: {}",
									content_id
								)
								.into(),
							);
						}

						let app_dir = app_handle
							.path()
							.app_local_data_dir()
							.expect("failed to get app data dir");
						let path = app_dir
							.join("content_blobs")
							.join(module_id.to_string())
							.join(&content.file_name);
						let blob = content_response.bytes().await.map_err(|e| e.to_string())?;
						std::fs::create_dir_all(app_dir.join("content_blobs").join(module_id.to_string()))
							.map_err(|e| e.to_string())?;
						std::fs::write(&path, blob).map_err(|e| e.to_string())?;

						let content_blob = entity::content_blob::ActiveModel {
							name: ActiveValue::Set(content.file_name.clone()),
							module_id: ActiveValue::Set(module_id),
							updated_at: ActiveValue::Set(Utc::now().timestamp()),
							mime_type: ActiveValue::Set(mime_type.to_string()),
							path: ActiveValue::Set(path.to_str().unwrap().to_string()),
						};

						entity::ContentBlob::insert(content_blob)
							.on_conflict(
								sea_query::OnConflict::columns([
									entity::content_blob::Column::Name,
									entity::content_blob::Column::ModuleId,
								])
								.update_columns([
									entity::content_blob::Column::ModuleId,
									entity::content_blob::Column::UpdatedAt,
									entity::content_blob::Column::MimeType,
									entity::content_blob::Column::Path,
								])
								.to_owned(),
							)
							.exec(&txn)
							.await
							.map_err(|e| e.to_string())?;

						// modules with the resource type usually (from what i've seen) only consist of a single content blob (pdf)
						// and so we set the module content to the content blob path
						if module.module_type == SectionModuleType::Resource {
							let module_content = entity::module_content::ActiveModel {
								id: ActiveValue::Set(content_id),
								module_id: ActiveValue::Set(module_id),
								content: ActiveValue::Set(content.file_name.to_string()),
								rank: ActiveValue::Set(i as i32),
								updated_at: ActiveValue::Set(Utc::now().timestamp()),
							};

							entity::ModuleContent::insert(module_content)
								.on_conflict(
									sea_query::OnConflict::columns([
										entity::module_content::Column::Id,
										entity::module_content::Column::ModuleId,
									])
									.update_columns([
										entity::module_content::Column::Content,
										entity::module_content::Column::Rank,
										entity::module_content::Column::UpdatedAt,
									])
									.to_owned(),
								)
								.exec(&txn)
								.await
								.map_err(|e| e.to_string())?;
						}
					}
				}

				// match module.module_type {
				// 	RestCourseSectionModuleType::Book => {
				// 		let structure_content = module_contents.iter().find(|content| {
				// 			content.content_type == RestCourseSectionModuleContentType::Content
				// 				&& content.file_name == "structure"
				// 		});

				// 		if let Some(structure_content) = structure_content {
				// 			let structure: Vec<rest::RestCourseSectionModuleStructureItem> =
				// 				serde_json::from_str(&structure_content.content.as_ref().unwrap())
				// 					.map_err(|e| e.to_string())?;

				// 			for item in structure
				// 				.iter()
				// 				.flat_map(|item| item.sub_items.iter().flatten())
				// 			{

				// 			}
				// 		}
				// 	}
				// }

				txn.commit().await.map_err(|e| e.to_string())?;
				Ok(())
			})
		})
		.await
		.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_user_courses(app: AppHandle) -> Result<Vec<entity::course::Model>, String> {
	SyncTask::new(app, "get_user_courses".to_string())
		.return_state(move |state| {
			let db = state.0.clone();
			Box::pin(async move {
				let courses = entity::Course::find()
					.all(&db)
					.await
					.map_err(|e| e.to_string())?;

				Ok(courses)
			})
		})
		.sync_state(|app_handle| {
			Box::pin(async move {
				{
					let store = app_handle.store("store.json").unwrap();
					let client = reqwest::Client::new();
					let user_id = store
						.get(auth_keys::USER_ID)
						.and_then(|id| id.as_str().and_then(|s| s.parse::<u32>().ok()))
						.ok_or("Failed to retrieve user id from store")?;

					let host = store.get(auth_keys::MOODLE_HOST).unwrap();
					let ws_token = store.get(auth_keys::WS_TOKEN).unwrap();
					let request = rest::get_user_courses_request(
						user_id,
						host.as_str().unwrap(),
						ws_token.as_str().unwrap(),
					)
					.map_err(|e| e.to_string())?;

					let response = client.execute(request).await.unwrap();
					if response.status().is_success().not() {
						return Err(
							format!(
								"Could not get user courses: {}",
								response.text().await.unwrap()
							)
							.into(),
						);
					}

					let body = response.text().await.unwrap();
					if body.contains("errorcode") {
						return Err(format!("Could not get user courses: {}", body).into());
					}

					let course_data =
						serde_json::from_str::<Vec<RestCourse>>(&body).map_err(|e| e.to_string())?;
					let courses = course_data
						.into_iter()
						.map(|course| entity::course::ActiveModel {
							id: ActiveValue::Set(course.id),
							name: ActiveValue::Set(course.full_name),
							colour: ActiveValue::Set(Some("brown".to_string())),
							icon: ActiveValue::Set(None),
						})
						.collect::<Vec<_>>();

					let state = app_handle.state::<DatabaseState>();
					let db = &state.0;
					let txn = db.begin().await.map_err(|e| e.to_string())?;

					for course in courses {
						entity::Course::insert(course)
							.on_conflict(
								sea_query::OnConflict::column(entity::course::Column::Id)
									.update_columns([
										entity::course::Column::Name,
										entity::course::Column::Colour,
										entity::course::Column::Icon,
									])
									.to_owned(),
							)
							.exec(&txn)
							.await
							.map_err(|e| e.to_string())?;
					}

					txn.commit().await.map_err(|e| e.to_string())?;
					Ok(())
				}
			})
		})
		.await
		.map_err(|e| e.to_string())
}
