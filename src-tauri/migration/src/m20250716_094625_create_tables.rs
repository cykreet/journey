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
enum CourseSectionItem {
	Table,
	Id,
	SectionId,
	Name,
	ContentType,
	UpdatedAt,
}

#[derive(DeriveIden)]
enum ModuleContent {
	Table,
	Id,
	UpdatedAt,
	Content,
}

#[derive(DeriveIden)]
enum ContentBlob {
	Table,
	Id,
	Name,
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
					.table(CourseSectionItem::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(CourseSectionItem::Id)
							.integer()
							.not_null()
							.primary_key(),
					)
					.col(
						ColumnDef::new(CourseSectionItem::SectionId)
							.integer()
							.not_null(),
					)
					.col(ColumnDef::new(CourseSectionItem::Name).string().not_null())
					.col(
						ColumnDef::new(CourseSectionItem::ContentType)
							.integer()
							// .enumeration(Alias::new("content_type"))
							.not_null()
							.unsigned(),
					)
					.col(
						ColumnDef::new(CourseSectionItem::UpdatedAt)
							.integer()
							.null(),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_course_section_item_section_id")
							.from(CourseSectionItem::Table, CourseSectionItem::SectionId)
							.to(CourseSection::Table, CourseSectionItem::Id)
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
					.col(
						ColumnDef::new(ModuleContent::Id)
							.integer()
							.not_null()
							.primary_key(),
					)
					.col(ColumnDef::new(ModuleContent::UpdatedAt).integer().null())
					.col(ColumnDef::new(ModuleContent::Content).text().null())
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(ContentBlob::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(ContentBlob::Id)
							.integer()
							.not_null()
							.primary_key(),
					)
					.col(ColumnDef::new(ContentBlob::Name).string().not_null())
					.col(ColumnDef::new(ContentBlob::MimeType).string().not_null())
					.col(ColumnDef::new(ContentBlob::Path).string().not_null())
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
			.drop_table(Table::drop().table(CourseSectionItem::Table).to_owned())
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
