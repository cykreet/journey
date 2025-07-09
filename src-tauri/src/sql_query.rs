use sqlx::{sqlite::SqliteQueryResult, Error, QueryBuilder, Sqlite};

use crate::entities::TableLike;

pub struct SqlQuery<'a> {
	builder: QueryBuilder<'a, Sqlite>,
	// values: Vec<&'a dyn sqlx::Encode<'a, Sqlite>>,
}

// inspired by kali (https://github.com/sylv/kali/tree/main), will probably
// replace this when its ready
impl<'a> SqlQuery<'a> {
	pub fn new() -> Self {
		return Self {
			builder: QueryBuilder::default(),
		};
	}

	pub fn select(mut self, table: String) -> Self {
		if self.builder.sql().len() > 0 {
			panic!("select can't be chained with previous queries");
		}

		let query_str = format!("SELECT * FROM {}", table);
		self.builder.push(query_str);
		self
	}

	pub fn where_column<T>(mut self, column: &str, value: &'a T) -> Self
	where
		T: sqlx::Encode<'a, Sqlite> + sqlx::Type<Sqlite>,
	{
		self.builder.push(format!(" WHERE {} = ", column));
		// pushes placeholder ? and binds value
		self.builder.push_bind(value);
		self
	}

	pub fn insert_into<T>(mut self, rows: &'a Vec<T>) -> Self
	where
		T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + TableLike,
	{
		if self.builder.sql().len() > 0 {
			panic!("insert_into can't be chained with previous queries");
		}

		// let values_per_row = column_names.len();
		// let placeholder_row = format!(
		// 	"({})",
		// 	(0..values_per_row)
		// 		.map(|_| "?")
		// 		.collect::<Vec<_>>()
		// 		.join(", ")
		// );

		// let placeholder_rows: Vec<String> = (0..rows.len()).map(|_| placeholder_row.clone()).collect();
		let table_name = T::table_name();
		let column_names = T::columns();
		let query_str = format!(
			"INSERT OR REPLACE INTO {} ({})",
			table_name,
			column_names.join(", "),
			// placeholder_rows.join(", ")
		);

		self.builder.push(query_str);
		// todo
		self.builder.push_values(rows.iter(), |b, row| {});

		self
	}

	pub async fn execute<'e, U>(self, executor: U) -> Result<SqliteQueryResult, Error>
	where
		U: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
	{
		let query = self.builder.sql();
		println!("query: {}", query);
		let query = sqlx::query(query);
		query.execute(executor).await
	}

	pub async fn fetch_one<'e, T, U>(self, executor: U) -> Result<T, Error>
	where
		T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + TableLike,
		U: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
	{
		let query = self.builder.sql();
		let query = sqlx::query(query);
		query
			.fetch_one(executor)
			.await
			.and_then(|row| T::from_row(&row))
	}

	pub async fn fetch_all<'e, T, U>(self, executor: U) -> Result<Vec<T>, Error>
	where
		T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + TableLike,
		U: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
	{
		let query = self.builder.sql();
		let query = sqlx::query(query);
		query.fetch_all(executor).await.and_then(|rows| {
			rows
				.into_iter()
				.map(|row| T::from_row(&row))
				.collect::<Result<Vec<_>, _>>()
		})
	}
}
