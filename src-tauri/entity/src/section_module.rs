use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(
	EnumIter, DeriveActiveEnum, Debug, DeriveDisplay, Serialize, Deserialize, PartialEq, Clone, Type,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum SectionModuleType {
	#[serde(rename = "page")]
	#[sea_orm(num_value = 0)]
	Page,
	#[serde(rename = "book")]
	#[sea_orm(num_value = 1)]
	Book,
	// todo: forums would have separate a forum_content (or similar) entity
	#[serde(rename = "forum")]
	#[sea_orm(num_value = 2)]
	Forum,
	#[serde(rename = "resource")]
	#[sea_orm(num_value = 3)]
	Resource,
	#[serde(rename = "url")]
	#[sea_orm(num_value = 4)]
	Url,
	#[serde(other)]
	#[sea_orm(num_value = -1)]
	Unknown,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Type, Serialize, Deserialize)]
#[sea_orm(table_name = "section_module")]
#[specta(rename = "SectionModule", rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub id: i32,
	pub section_id: i32,
	pub name: String,
	pub updated_at: i64,
	pub module_type: SectionModuleType,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::course_section::Entity",
		from = "Column::SectionId",
		to = "super::course_section::Column::Id"
	)]
	Section,
	#[sea_orm(
		has_many = "super::module_content::Entity",
		from = "Column::Id",
		to = "super::module_content::Column::ModuleId"
	)]
	ModuleContent,
	#[sea_orm(
		has_many = "super::content_blob::Entity",
		from = "Column::Id",
		to = "super::content_blob::Column::ModuleId"
	)]
	ContentBlob,
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

impl Related<super::content_blob::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::ContentBlob.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
