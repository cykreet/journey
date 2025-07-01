extern crate proc_macro;

use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::{FromRow, Sqlite};

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

// impl<'q> Encode<'q, Sqlite> for ContentType {
// 	fn encode_by_ref(
// 		&self,
// 		buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
// 	) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
// 		<Json<&ContentType> as Encode<Sqlite>>::encode(Json(self), buf)
// 	}
// }

// impl<'q> Decode<'q, Sqlite> for ContentType {
// 	fn decode(
// 		value: <Sqlite as sqlx::Database>::ValueRef<'q>,
// 	) -> Result<Self, sqlx::error::BoxDynError> {
// 		let Json(content_type) = <Json<ContentType> as Decode<Sqlite>>::decode(value)?;
// 		Ok(content_type)
// 	}
// }

// impl sqlx::Type<Sqlite> for ContentType {}

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
pub struct CourseItem {
	id: u32,
	parent_id: Option<u32>,
	#[sqlx(skip)]
	sync_hash: u64,
	course_id: u32,
	name: String,
	content_type: ContentType,
	updated_at: String,
	content: String,
}

impl TableLike for CourseItem {
	fn table_name() -> String {
		"course_item".to_string()
	}

	fn column_names() -> Vec<String> {
		vec![
			"id".to_string(),
			"parent_id".to_string(),
			"course_id".to_string(),
			"name".to_string(),
			"content_type".to_string(),
			"updated_at".to_string(),
			"content".to_string(),
		]
	}

	fn bind_to_query<'q>(
		&'q self,
		query: sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
	) -> sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
		query.bind(&self.id).bind(&self.course_id).bind(&self.name)
	}
}

impl Hash for CourseItem {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
		self.content.hash(state);
	}
}
