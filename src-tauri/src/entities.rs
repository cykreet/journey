extern crate proc_macro;

use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Type)]
pub enum ContentType {
	Markup,
	Resource,
}

#[derive(Serialize, Deserialize, Type, FromRow)]
pub struct Course {
	pub id: u32,
	pub name: String,
	pub colour: Option<String>,
	pub icon: Option<String>,
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

#[derive(Serialize, Deserialize, Type, FromRow)]
pub struct CourseItem {
	id: u32,
	course_id: u32,
	name: String,
	content: CourseItemContent,
	children: Option<Vec<CourseItem>>,
}

impl Hash for CourseItemContent {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.title.hash(state);
		self.content.hash(state);
	}
}
