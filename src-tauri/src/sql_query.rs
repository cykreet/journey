use kali::builder::value::Value;
use sqlx::{sqlite::SqliteQueryResult, Error, QueryBuilder, Sqlite};

use crate::entities::TableLike;

pub struct SqlQuery<'a> {
	builder: QueryBuilder<'a, Sqlite>,
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

	pub fn where_column<T>(mut self, column: &str, value: T) -> Self
	where
		T: Into<Value>,
	{
		self.builder.push(format!(" WHERE {} = ", column));
		let value: Value = value.into();
		match value {
			// pushes placeholder ? and binds value
			Value::Bool(v) => self.builder.push_bind(v),
			Value::String(v) => self.builder.push_bind(v),
			Value::Integer(v) => self.builder.push_bind(v),
			Value::Real(v) => self.builder.push_bind(v),
			Value::Blob(v) => self.builder.push_bind(v),
			Value::Null => self.builder.push_bind::<Option<i32>>(None),
		};
		self
	}

	pub fn insert_into<T>(mut self, rows: &Vec<T>) -> Self
	where
		T: TableLike,
	{
		if !self.builder.sql().is_empty() {
			panic!("insert_into can't be chained with previous queries");
		}

		let table_name = T::table_name();
		let column_names = T::columns();
		let query_str = format!(
			"INSERT OR REPLACE INTO {} ({})",
			table_name,
			column_names.join(", ")
		);

		self.builder.push(query_str);
		self.builder.push(" VALUES ");
		for (i, row) in rows.iter().enumerate() {
			if i > 0 {
				self.builder.push(", ");
			}

			let values = row.to_values();
			self.builder.push("(");
			for (j, value) in values.into_iter().enumerate() {
				if j > 0 {
					self.builder.push(", ");
				}

				match value {
					Value::Bool(v) => self.builder.push_bind(v),
					Value::String(v) => self.builder.push_bind(v),
					Value::Integer(v) => self.builder.push_bind(v),
					Value::Real(v) => self.builder.push_bind(v),
					Value::Blob(v) => self.builder.push_bind(v),
					Value::Null => self.builder.push_bind::<Option<i32>>(None),
				};
			}

			self.builder.push(")");
		}

		self
	}

	pub async fn execute<'e, U>(mut self, executor: U) -> Result<SqliteQueryResult, Error>
	where
		U: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
	{
		self.builder.build().execute(executor).await
	}

	pub async fn fetch_one<'e, T, U>(mut self, executor: U) -> Result<T, Error>
	where
		T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + TableLike,
		U: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
	{
		self
			.builder
			.build()
			.fetch_one(executor)
			.await
			.and_then(|row| T::from_row(&row))
	}

	pub async fn fetch_all<'e, T, U>(mut self, executor: U) -> Result<Vec<T>, Error>
	where
		T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + TableLike,
		U: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
	{
		self
			.builder
			.build()
			.fetch_all(executor)
			.await
			.and_then(|rows| {
				rows
					.into_iter()
					.map(|row| T::from_row(&row))
					.collect::<Result<Vec<_>, _>>()
			})
	}
}
