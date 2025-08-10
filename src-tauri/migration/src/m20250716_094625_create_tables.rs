use sea_orm_migration::{prelude::*, sea_orm};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Course {
	Table,
	Id,
	Name,
	Colour,
	Icon,
}

#[derive(DeriveIden)]
enum CourseSection {
	Table,
	Id,
	CourseId,
	Name,
}

#[derive(DeriveIden)]
enum SectionModule {
	Table,
	Id,
	SectionId,
	Name,
	UpdatedAt,
	ModuleType,
}

#[derive(DeriveIden)]
enum ModuleContent {
	Table,
	Id,
	ModuleId,
	UpdatedAt,
	Rank,
	Content,
}

#[derive(DeriveIden)]
enum ContentBlob {
	Table,
	Name,
	ModuleId,
	UpdatedAt,
	MimeType,
	Path,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Course::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Course::Id)
							.integer()
							.not_null()
							.primary_key(),
					)
					.col(ColumnDef::new(Course::Name).string().not_null())
					.col(ColumnDef::new(Course::Colour).string().null())
					.col(ColumnDef::new(Course::Icon).string().null())
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(CourseSection::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(CourseSection::Id)
							.integer()
							.not_null()
							.primary_key(),
					)
					.col(ColumnDef::new(CourseSection::CourseId).integer().not_null())
					.col(ColumnDef::new(CourseSection::Name).string().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_course_section_course_id")
							.from(CourseSection::Table, CourseSection::CourseId)
							.to(Course::Table, Course::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(SectionModule::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(SectionModule::Id)
							.integer()
							.not_null()
							.primary_key(),
					)
					.col(
						ColumnDef::new(SectionModule::SectionId)
							.integer()
							.not_null(),
					)
					.col(ColumnDef::new(SectionModule::Name).string().not_null())
					.col(ColumnDef::new(SectionModule::UpdatedAt).integer().null())
					.col(
						ColumnDef::new(SectionModule::ModuleType)
							.integer()
							.not_null(),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_course_section_item_section_id")
							.from(SectionModule::Table, SectionModule::SectionId)
							.to(CourseSection::Table, SectionModule::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(ModuleContent::Table)
					.if_not_exists()
					.col(ColumnDef::new(ModuleContent::Id).integer().not_null())
					.col(ColumnDef::new(ModuleContent::ModuleId).integer().not_null())
					.primary_key(
						Index::create()
							.col(ModuleContent::Id)
							.col(ModuleContent::ModuleId),
					)
					.col(ColumnDef::new(ModuleContent::UpdatedAt).integer().null())
					.col(ColumnDef::new(ModuleContent::Rank).integer().not_null())
					.col(ColumnDef::new(ModuleContent::Content).string().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_module_content_section_module_id")
							.from(ModuleContent::Table, ModuleContent::ModuleId)
							.to(SectionModule::Table, SectionModule::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(ContentBlob::Table)
					.if_not_exists()
					.col(ColumnDef::new(ContentBlob::Name).string().not_null())
					.col(ColumnDef::new(ContentBlob::ModuleId).integer().not_null())
					.primary_key(
						Index::create()
							.col(ContentBlob::Name)
							.col(ContentBlob::ModuleId),
					)
					.col(ColumnDef::new(ContentBlob::UpdatedAt).integer().null())
					.col(ColumnDef::new(ContentBlob::MimeType).string().not_null())
					.col(ColumnDef::new(ContentBlob::Path).string().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_content_blob_module_id")
							.from(ContentBlob::Table, ContentBlob::ModuleId)
							.to(SectionModule::Table, SectionModule::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Course::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(CourseSection::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(SectionModule::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(ModuleContent::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(ContentBlob::Table).to_owned())
			.await?;

		Ok(())
	}
}
