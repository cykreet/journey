extern crate proc_macro;

use std::hash::{Hash, Hasher};

use kali::builder::value::Value;
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::{types::Json, FromRow};

pub trait TableLike {
	fn table_name() -> String;
	fn columns() -> Vec<String>;
	fn to_values(&self) -> Vec<Value>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, sqlx::Type)]
pub enum ContentType {
	Page,
	Forum,
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

	fn columns() -> Vec<String> {
		vec![
			"id".to_string(),
			"name".to_string(),
			"colour".to_string(),
			"icon".to_string(),
		]
	}

	fn to_values(&self) -> Vec<Value> {
		vec![
			self.id.into(),
			self.name.clone().into(),
			self.colour.clone().into(),
			self.icon.clone().into(),
		]
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
	pub updated_at: Option<u32>,
	// #[sqlx(skip)]
	// pub sync_hash: u64,
}

impl TableLike for CourseSection {
	fn table_name() -> String {
		"course_section".to_string()
	}

	fn columns() -> Vec<String> {
		vec![
			"id".to_string(),
			"name".to_string(),
			"course_id".to_string(),
			"items".to_string(),
		]
	}

	fn to_values(&self) -> Vec<Value> {
		vec![
			self.id.into(),
			self.name.clone().into(),
			self.course_id.into(),
			serde_json::to_string(&self.items.0)
				.unwrap_or_else(|_| "[]".to_string())
				.into(),
		]
	}
}

// impl Hash for CourseItem {
// 	fn hash<H: Hasher>(&self, state: &mut H) {
// 		self.name.hash(state);
// 		self.content.hash(state);
// 	}
// }
