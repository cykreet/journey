use std::{ops::Not, vec};

use anyhow::{Context, anyhow};
use entity::section_module::SectionModuleType;
use migration::Expr;
use sea_orm::{
	ActiveValue, ColumnTrait, Condition, EntityTrait, QueryFilter, TransactionTrait, sea_query,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::types::chrono::Utc;
use tauri::{AppHandle, Manager};
use tauri_plugin_http::reqwest;
use tauri_plugin_store::StoreExt;

use entity::content_blob::Model as ContentBlob;
use entity::course::Model as Course;
use entity::course_section::Model as CourseSection;
use entity::module_content::Model as ModuleContent;
use entity::section_module::Model as SectionModule;

use crate::{
	auth::auth_keys,
	database::DatabaseState,
	request::rest::{self, RestCourse, RestCourseSection},
	sync_task::{SyncError, SyncTask},
};

#[derive(Serialize, Deserialize, Type)]
pub struct CourseWithSections {
	pub course: Course,
	pub sections: Vec<CourseSectionWithModules>,
}

#[derive(Serialize, Deserialize, Type)]
pub struct CourseSectionWithModules {
	pub section: CourseSection,
	pub modules: Vec<SectionModule>,
}

// we filter for these when revalidating and fetching course data
pub const SUPPORTED_MODULE_TYPES: [SectionModuleType; 3] = [
	SectionModuleType::Page,
	SectionModuleType::Book,
	SectionModuleType::Resource,
];

// supported mime types relevant to actual module content and not embedded content like images
pub const SUPPORTED_RESOURCE_TYPES: [&str; 1] = ["application/pdf"];

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
					.await?
					.into_iter()
					.next()
					.ok_or_else(|| format!("Course with id {} not found", course_id))?;

				let mut sections_with_items = vec![];
				for section in sections {
					let modules = entity::SectionModule::find()
						.filter(
							Condition::all()
								.add(entity::section_module::Column::SectionId.eq(section.id))
								.add(entity::section_module::Column::ModuleType.is_in(SUPPORTED_MODULE_TYPES)),
						)
						.all(&db)
						.await?;

					let supported_modules = modules
						.into_iter()
						.filter(|module| {
							if module.module_type == SectionModuleType::Resource {
								let mime_types: Vec<String> =
									serde_json::from_value(module.mime_types.clone().unwrap_or_default())
										.unwrap_or_default();
								return SUPPORTED_RESOURCE_TYPES
									.contains(&mime_types.first().unwrap_or(&"".to_string()).as_str());
							}

							true
						})
						.collect::<Vec<_>>();

					if supported_modules.is_empty().not() {
						sections_with_items.push((section, supported_modules));
					}
				}

				Ok(CourseWithSections {
					course: course,
					sections: sections_with_items
						.into_iter()
						.map(|(section, modules)| CourseSectionWithModules { section, modules })
						.collect(),
				})
			})
		})
		.sync_state(move |app_handle| {
			Box::pin(async move {
				let store = app_handle.store("store.json").unwrap();
				let host = store.get(auth_keys::MOODLE_HOST).unwrap();
				let client = reqwest::Client::new();
				let ws_token = store.get(auth_keys::WS_TOKEN).unwrap();
				let request = rest::get_course_sections_request(
					&client,
					host.as_str().unwrap(),
					ws_token.as_str().unwrap(),
					course_id,
				)
				.map_err(|e| anyhow!("Failed to create request: {}", e))?;

				let response = client
					.execute(request)
					.await
					.map_err(|e| anyhow!("Failed to execute request for course sections: {}", e))?;
				if response.status().is_success().not() {
					return Err(SyncError::from(anyhow!(
						"Failed to fetch course sections with id: {course_id}"
					)));
				}

				let body = response
					.text()
					.await
					.map_err(|e| anyhow!("Failed to read response body: {}", e))?;
				if body.contains("errorcode") {
					let error_body: rest::RestErrorBody =
						serde_json::from_str(&body).with_context(|| "Failed to parse error body")?;

					return Err(SyncError {
						code: Some(error_body.error_code),
						message: error_body.message,
					});
				}

				let sections_data: Vec<RestCourseSection> =
					serde_json::from_str(&body).with_context(|| "Failed to parse sections data")?;
				let state = app_handle.state::<DatabaseState>();
				let db = &state.0;
				let txn = db
					.begin()
					.await
					.map_err(|e| anyhow!("Failed to begin transaction for course sections: {}", e))?;

				entity::Course::update_many()
					// module_count helps us keep track of the number of modules in this course
					// so we can be more transparent about any that are omitted when we wanna display them
					.col_expr(
						entity::course::Column::ModuleCount,
						Expr::value(
							sections_data
								.iter()
								.map(|section| section.modules.len() as i32)
								.sum::<i32>(),
						),
					)
					.filter(entity::course::Column::Id.eq(course_id))
					.exec(&txn)
					.await
					.map_err(|e| anyhow!("Failed to update course module count: {}", e))?;

				for section in sections_data {
					let section_entity = entity::course_section::ActiveModel {
						id: ActiveValue::Set(section.id),
						name: ActiveValue::Set(html_escape::decode_html_entities(&section.name).to_string()),
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
						.map_err(|e| anyhow!("Failed to insert course section {}: {}", section.id, e))?;

					for module in section.modules {
						if !SUPPORTED_MODULE_TYPES.contains(&module.module_type) {
							continue;
						}

						let section_item = entity::section_module::ActiveModel {
							id: ActiveValue::Set(module.id),
							name: ActiveValue::Set(html_escape::decode_html_entities(&module.name).to_string()),
							section_id: ActiveValue::Set(section.id),
							module_type: ActiveValue::Set(module.module_type),
							mime_types: match module.contents_info {
								Some(contents_info) => ActiveValue::Set(Some(
									serde_json::to_value(contents_info.mime_types).map_err(|e| {
										anyhow!(
											"Failed to serialize mime types for module {}: {}",
											module.id,
											e
										)
									})?,
								)),
								None => ActiveValue::NotSet,
							},
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
							.map_err(|e| anyhow!("Failed to insert section module {}: {}", module.id, e))?;
					}
				}

				txn
					.commit()
					.await
					.map_err(|e| anyhow!("Failed to commit transaction for course sections: {}", e))?;
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
) -> Result<(SectionModule, Vec<ModuleContent>), String> {
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
					&client,
					store.get(auth_keys::MOODLE_HOST).unwrap().as_str().unwrap(),
					token,
					course_id,
					module_id,
				)
				.map_err(|e| anyhow!("Failed to create request: {}", e))?;

				let response = client
					.execute(request)
					.await
					.map_err(|e| anyhow!("Failed to execute request for module content: {}", e))?;
				if response.status().is_success().not() {
					return Err(SyncError::from(anyhow!(
						"Failed to fetch module content with id: {module_id}"
					)));
				}

				let body = response
					.text()
					.await
					.with_context(|| "Failed to read response body")?;
				if body.contains("errorcode") {
					let error_body: rest::RestErrorBody =
						serde_json::from_str(&body).with_context(|| "Failed to parse error body")?;

					println!(
						"course id {course_id}, module id {module_id}, error: {:?}",
						error_body
					);
					return Err(SyncError {
						code: Some(error_body.error_code),
						message: error_body.message,
					});
				}

				let sections_data: Vec<RestCourseSection> = serde_json::from_str(&body)
					.map_err(|e| anyhow!("Failed to parse sections data: {}", e))?;
				let module = sections_data
					.into_iter()
					.flat_map(|section| section.modules)
					.find(|module| module.id == module_id)
					.with_context(|| format!("Module with id {} not found", module_id))?;

				if SUPPORTED_MODULE_TYPES.contains(&module.module_type).not() {
					return Err(SyncError::from(anyhow!(
						"Module type {} is not supported",
						module.module_type
					)));
				}

				let module_contents = module.contents.unwrap_or_default();
				let state = app_handle.state::<DatabaseState>();
				let db = &state.0;
				let txn = db
					.begin()
					.await
					.map_err(|e| anyhow!("Failed to begin transaction for module content: {}", e))?;

				for (i, content) in module_contents.iter().enumerate() {
					// ids of the content blocks stored in file path as "/id/"
					// media content also uses this to refer to the relevant content block.
					// the "root" content block seems to always have a path of "/", which is usually
					// included if it doesn't contain any other content

					// "books" have an additional structure content object that contains the hierarchy of the contents,
					// not sure how i wanna handle books, but storing content blocks as they appear in the response is fine for now
					let content_id = if content.file_path == "/" && content.file_name != "structure" {
						1
					} else {
						if module.module_type == SectionModuleType::Book && content.file_name == "structure" {
							0
						} else {
							content.file_path[1..content.file_path.len() - 1]
								.parse::<i32>()
								.map_err(|e| {
									anyhow!(
										"Failed to parse content id from file path {}: {}",
										content.file_path,
										e
									)
								})?
						}
					};

					// written content is usually in an index.html file. we generally wanna store text content directly, blobs being
					// stored on the filesystem, with paths stored in the database.
					if content.file_name == "index.html" {
						let existing_content = entity::ModuleContent::find()
							.filter(
								Condition::all()
									.add(entity::module_content::Column::ModuleId.eq(module_id))
									.add(entity::module_content::Column::Id.eq(content_id)),
							)
							.one(&txn)
							.await
							.map_err(|e| {
								anyhow!(
									"Failed to query existing module content {}: {}",
									content.file_name,
									e
								)
							})?;

						if let Some(module_content) = existing_content
							&& content.time_modified as i64 > module_content.updated_at
						{
							continue;
						}

						let file_url = content.file_url.as_ref().with_context(|| {
							format!("Content with id {} does not have a file URL", content_id)
						})?;
						let file_url = format!("{}?forcedownload=1&token={}", file_url, token);
						let content_response = reqwest::get(file_url).await.map_err(|e| {
							anyhow!(
								"Failed to fetch content for content id {}: {}",
								content_id,
								e
							)
						})?;
						if content_response.status().is_success().not() {
							return Err(SyncError::from(anyhow!(
								"Failed to fetch content for content id: {}",
								content_id
							)));
						}

						// todo: remove any scripts and stylesheets that are not needed
						// html content is usually also pretty ugly with empty tags, etc.
						let content_text = content_response.text().await.map_err(|e| {
							anyhow!(
								"Failed to read content response for content id {}: {}",
								content_id,
								e
							)
						})?;
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
							.map_err(|e| anyhow!("Failed to insert module content: {}", e))?;
					}

					if let Some(mime_type) = &content.mime_type {
						let app_dir = app_handle
							.path()
							.app_local_data_dir()
							.expect("failed to get app data dir");
						let path = app_dir
							.join("content_blobs")
							.join(module_id.to_string())
							.join(&content.file_name);

						let file_exists = std::fs::exists(&path).map_err(|e| {
							anyhow!(
								"Failed to check if content blob file exists {}: {}",
								path.to_str().unwrap_or_default(),
								e
							)
						})?;
						let existing_blob = entity::ContentBlob::find()
							.filter(
								Condition::all()
									.add(entity::content_blob::Column::ModuleId.eq(module_id))
									.add(entity::content_blob::Column::Name.eq(&content.file_name)),
							)
							.one(&txn)
							.await
							.map_err(|e| {
								anyhow!(
									"Failed to query existing content blob {}: {}",
									content.file_name,
									e
								)
							})?;

						// if the file exists on disk and hasn't been updated, we shouldn't need to download it again
						if file_exists
							&& let Some(blob) = existing_blob
							&& content.time_modified as i64 > blob.updated_at
						{
							continue;
						}

						let file_url = content.file_url.as_ref().with_context(|| {
							format!("Content with id {} does not have a file URL", content_id)
						})?;
						let file_url = format!("{}?forcedownload=1&token={}", file_url, token);
						let content_response = reqwest::get(file_url).await.map_err(|e| {
							anyhow!(
								"Failed to fetch content blob for content id {}: {}",
								content_id,
								e
							)
						})?;
						if content_response.status().is_success().not() {
							return Err(SyncError::from(anyhow!(
								"Failed to fetch content blob for content id: {}",
								content_id
							)));
						}

						let blob = content_response.bytes().await.map_err(|e| {
							anyhow!(
								"Failed to read content blob response for content id {}: {}",
								content_id,
								e
							)
						})?;
						std::fs::create_dir_all(app_dir.join("content_blobs").join(module_id.to_string()))
							.map_err(|e| {
								anyhow!(
									"Failed to create content blob directory {}: {}",
									path.to_str().unwrap_or_default(),
									e
								)
							})?;
						std::fs::write(&path, blob).map_err(|e| {
							anyhow!(
								"Failed to write content blob to file {}: {}",
								path.to_str().unwrap_or_default(),
								e
							)
						})?;

						let content_blob = entity::content_blob::ActiveModel {
							name: ActiveValue::Set(content.file_name.clone()),
							module_id: ActiveValue::Set(module_id),
							// updated_at: ActiveValue::Set(Utc::now().timestamp()),
							updated_at: ActiveValue::Set(
								content
									.time_modified
									.try_into()
									.unwrap_or(Utc::now().timestamp()),
							),
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
							.map_err(|e| anyhow!("Failed to insert content blob: {}", e))?;

						// modules with the resource type usually (from what i've seen) only consist of a single content blob (pdf)
						// and so we set the module content to the content blob path
						if module.module_type == SectionModuleType::Resource
							&& SUPPORTED_RESOURCE_TYPES.contains(&mime_type.as_str())
						{
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
								.map_err(|e| anyhow!("Failed to insert module content: {}", e))?;
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
				// 					?;

				// 			for item in structure
				// 				.iter()
				// 				.flat_map(|item| item.sub_items.iter().flatten())
				// 			{

				// 			}
				// 		}
				// 	}
				// }

				txn
					.commit()
					.await
					.map_err(|e| anyhow!("Failed to commit transaction: {}", e))?;
				Ok(())
			})
		})
		.await
		.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_content_blobs(
	app: AppHandle,
	course_id: i32,
	module_id: i32,
) -> Result<Vec<ContentBlob>, String> {
	SyncTask::new(
		app,
		format!("get_content_blobs_{}_{}", course_id, module_id),
	)
	.return_state(move |state| {
		let db = state.0.clone();
		Box::pin(async move {
			let blobs = entity::ContentBlob::find()
				.filter(Condition::all().add(entity::content_blob::Column::ModuleId.eq(module_id)))
				.all(&db)
				.await?;

			if blobs.is_empty() {
				return Err(format!("No content blobs found for module id: {}", module_id).into());
			}

			Ok(blobs)
		})
	})
	.sync_state(|_| Box::pin(async { Ok(()) }))
	.await
	.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_user_courses(app: AppHandle) -> Result<Vec<Course>, String> {
	SyncTask::new(app, "get_user_courses".to_string())
		.return_state(move |state| {
			let db = state.0.clone();
			Box::pin(async move {
				let courses = entity::Course::find().all(&db).await?;
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
						.with_context(|| "Failed to retrieve user id from store")
						.map_err(|e| SyncError::from(anyhow!("Failed to get user id: {}", e)))?;

					let host = store.get(auth_keys::MOODLE_HOST).unwrap();
					let ws_token = store.get(auth_keys::WS_TOKEN).unwrap();
					let request = rest::get_user_courses_request(
						&client,
						ws_token.as_str().unwrap(),
						host.as_str().unwrap(),
						user_id,
					)
					.map_err(|e| SyncError::from(anyhow!("Failed to create request: {}", e)))?;

					let response = client
						.execute(request)
						.await
						.map_err(|e| SyncError::from(anyhow!("Failed to execute request: {}", e)))?;
					if response.status().is_success().not() {
						return Err(SyncError::from(anyhow!(
							"Could not get user courses: {}",
							response.text().await.unwrap_or_default()
						)));
					}

					let body = response
						.text()
						.await
						.with_context(|| "Failed to read response body")?;
					if body.contains("errorcode") {
						let error_body: rest::RestErrorBody =
							serde_json::from_str(&body).with_context(|| "Failed to parse error body")?;

						return Err(SyncError {
							code: Some(error_body.error_code),
							message: error_body.message,
						});
					}

					let course_data = serde_json::from_str::<Vec<RestCourse>>(&body)
						.with_context(|| "Failed to parse course data")?;
					let courses = course_data
						.into_iter()
						.map(|course| entity::course::ActiveModel {
							id: ActiveValue::Set(course.id),
							name: ActiveValue::Set(course.full_name),
							colour: ActiveValue::Set(Some("brown".to_string())),
							module_count: ActiveValue::Set(0),
							icon: ActiveValue::Set(None),
						})
						.collect::<Vec<_>>();

					let state = app_handle.state::<DatabaseState>();
					let db = &state.0;
					let txn = db
						.begin()
						.await
						.map_err(|e| SyncError::from(anyhow!("Failed to begin transaction: {}", e)))?;

					for course in courses {
						entity::Course::insert(course)
							.on_conflict(
								sea_query::OnConflict::column(entity::course::Column::Id)
									.update_columns([
										entity::course::Column::Name,
										entity::course::Column::Colour,
										entity::course::Column::Icon,
										// don't update module count here
									])
									.to_owned(),
							)
							.exec(&txn)
							.await
							.map_err(|e| SyncError::from(anyhow!("Failed to insert course: {}", e)))?;
					}

					txn
						.commit()
						.await
						.map_err(|e| SyncError::from(anyhow!("Failed to commit transaction: {}", e)))?;
					Ok(())
				}
			})
		})
		.await
		.map_err(|e| e.to_string())
}
