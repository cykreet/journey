use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(EnumIter, DeriveActiveEnum, Type, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum ContentType {
	#[sea_orm(num_value = 0)]
	Page,
	#[sea_orm(num_value = 1)]
	Forum,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Type, Serialize, Deserialize)]
#[sea_orm(table_name = "course_section_item")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub section_id: i32,
	pub name: String,
	pub content_type: ContentType,
	pub updated_at: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::course_section::Entity",
		from = "Column::SectionId",
		to = "super::course_section::Column::Id"
	)]
	Section,
	#[sea_orm(has_one = "super::module_content::Entity")]
	ModuleContent,
}

impl Related<super::course_section::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Section.def()
	}
}

impl Related<super::module_content::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::ModuleContent.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
