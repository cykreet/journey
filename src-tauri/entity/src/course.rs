use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Type, Serialize, Deserialize)]
#[sea_orm(table_name = "course")]
#[specta(rename = "Course")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub id: i32,
	pub name: String,
	pub colour: Option<String>,
	pub icon: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(has_many = "super::course_section::Entity")]
	CourseSection,
}

impl Related<super::course_section::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::CourseSection.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
