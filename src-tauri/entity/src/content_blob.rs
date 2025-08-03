use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Type, Serialize, Deserialize)]
#[sea_orm(table_name = "content_blob")]
#[specta(rename = "ContentBlob")]
pub struct Model {
	// #[sea_orm(primary_key)]
	// pub id: i32,
	#[sea_orm(primary_key)]
	pub name: String,
	#[sea_orm(primary_key, auto_increment = false)]
	pub module_content_id: i32,
	pub updated_at: i64,
	pub mime_type: String,
	pub path: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::module_content::Entity",
		from = "Column::ModuleContentId",
		to = "super::module_content::Column::Id"
	)]
	ModuleContent,
}

impl Related<super::module_content::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::ModuleContent.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
