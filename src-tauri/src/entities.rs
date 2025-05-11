extern crate proc_macro;

use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::{types::Json, Decode, Encode, FromRow, Sqlite};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum ContentType {
	Markup,
	Resource,
}

impl<'q> Encode<'q, Sqlite> for ContentType {
	fn encode_by_ref(
		&self,
		buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
	) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
		<Json<&ContentType> as Encode<Sqlite>>::encode(Json(self), buf)
	}
}

impl<'q> Decode<'q, Sqlite> for ContentType {
	fn decode(
		value: <Sqlite as sqlx::Database>::ValueRef<'q>,
	) -> Result<Self, sqlx::error::BoxDynError> {
		let Json(content_type) = <Json<ContentType> as Decode<Sqlite>>::decode(value)?;
		Ok(content_type)
	}
}

#[derive(Serialize, Deserialize, Type, FromRow)]
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

	fn to_values(&self) -> Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send>> {
		vec![
			Box::new(self.id),
			Box::new(self.name.clone()),
			Box::new(self.colour.clone()),
			Box::new(self.icon.clone()),
		]
	}
}

#[derive(Serialize, Deserialize, Type, FromRow)]
pub struct CourseItemContent {
	id: u32,
	#[sqlx(skip)]
	sync_hash: u64,
	course_id: u32,
	title: String,
	content_type: ContentType,
	content: Option<String>,
}

impl TableLike for CourseItemContent {
	fn table_name() -> String {
		"course_content".to_string()
	}

	fn column_names() -> Vec<String> {
		vec![
			"id".to_string(),
			"course_id".to_string(),
			"title".to_string(),
			"content_type".to_string(),
			"content".to_string(),
		]
	}

	fn to_values(&self) -> Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send>> {
		vec![
			Box::new(self.id),
			Box::new(self.course_id),
			Box::new(self.title.clone()),
			Box::new(self.content_type.clone()),
			Box::new(self.content.clone()),
		]
	}
}

#[derive(Serialize, Deserialize, Type, FromRow)]
pub struct CourseItem {
	id: u32,
	course_id: u32,
	name: String,
	content: CourseItemContent,
	children: Option<Vec<CourseItem>>,
}

impl TableLike for CourseItem {
	fn table_name() -> String {
		"course_item".to_string()
	}

	fn column_names() -> Vec<String> {
		vec![
			"id".to_string(),
			"course_id".to_string(),
			"name".to_string(),
		]
	}

	fn to_values(&self) -> Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send>> {
		vec![
			Box::new(self.id),
			Box::new(self.course_id),
			Box::new(self.name.clone()),
		]
	}
}

impl Hash for CourseItemContent {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.title.hash(state);
		self.content.hash(state);
	}
}

pub trait TableLike {
	fn table_name() -> String;
	fn column_names() -> Vec<String>;
	fn to_values(&self) -> Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send>>;
}
