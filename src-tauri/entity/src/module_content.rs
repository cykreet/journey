use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Type, Serialize, Deserialize)]
#[sea_orm(table_name = "module_content")]
#[specta(rename = "ModuleContent", rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub id: i32,
	#[sea_orm(primary_key, auto_increment = false)]
	pub module_id: i32,
	pub updated_at: i64,
	pub rank: i32,
	pub content: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::section_module::Entity",
		from = "Column::ModuleId",
		to = "super::section_module::Column::Id"
	)]
	SectionModule,
}

impl Related<super::section_module::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::SectionModule.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
