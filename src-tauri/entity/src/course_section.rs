use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Type, Serialize, Deserialize)]
#[sea_orm(table_name = "course_section")]
#[specta(rename = "CourseSection")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub course_id: i32,
	pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::course::Entity",
		from = "Column::CourseId",
		to = "super::course::Column::Id"
	)]
	Course,
	#[sea_orm(has_many = "super::course_section_item::Entity")]
	CourseSectionItem,
}

impl Related<super::course::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Course.def()
	}
}

impl Related<super::course_section_item::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::CourseSectionItem.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
