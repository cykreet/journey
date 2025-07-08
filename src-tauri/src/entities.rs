extern crate proc_macro;

use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::{types::Json, FromRow, Sqlite};

pub trait TableLike {
	fn table_name() -> String;
	fn column_names() -> Vec<String>;
	fn bind_to_query<'q>(
		&'q self,
		query: sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
	) -> sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, sqlx::Type)]
pub enum ContentType {
	Markup,
	Resource,
}

#[derive(Serialize, Deserialize, Type, FromRow, Clone)]
pub struct Course {
	pub id: u32,
	pub name: String,
	pub colour: Option<String>,
	pub icon: Option<String>,
}

impl TableLike for Course {
	fn table_name() -> String {
		"course".to_string()
	}

	fn column_names() -> Vec<String> {
		vec![
			"id".to_string(),
			"name".to_string(),
			"colour".to_string(),
			"icon".to_string(),
		]
	}

	fn bind_to_query<'q>(
		&'q self,
		query: sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
	) -> sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
		query
			.bind(&self.id)
			.bind(&self.name)
			.bind(&self.colour)
			.bind(&self.icon)
	}
}

#[derive(Serialize, Deserialize, Type, FromRow, Clone)]
pub struct CourseSection {
	pub id: u32,
	pub name: String,
	pub course_id: u32,
	#[specta(type = Vec<CourseSectionItem>)]
	pub items: Json<Vec<CourseSectionItem>>,
}

#[derive(Serialize, Deserialize, Type, Clone, sqlx::Type)]
pub struct CourseSectionItem {
	pub id: u32,
	pub name: String,
	pub content_type: ContentType,
	// #[sqlx(skip)]
	// sync_hash: u64,
	// updated_at: String,
}

impl TableLike for CourseSection {
	fn table_name() -> String {
		"course_section".to_string()
	}

	fn column_names() -> Vec<String> {
		vec![
			"id".to_string(),
			"name".to_string(),
			"course_id".to_string(),
			"items".to_string(),
		]
	}

	fn bind_to_query<'q>(
		&'q self,
		query: sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
	) -> sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
		query
			.bind(&self.id)
			.bind(&self.name)
			.bind(&self.course_id)
			.bind(&self.items)
	}
}

// impl Hash for CourseItem {
// 	fn hash<H: Hasher>(&self, state: &mut H) {
// 		self.name.hash(state);
// 		self.content.hash(state);
// 	}
// }
