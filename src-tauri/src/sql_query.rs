use kali::builder::value::Value;
use sqlx::{sqlite::SqliteQueryResult, Error, QueryBuilder, Sqlite};

use crate::entities::TableLike;

pub struct SqlQuery<'a> {
	builder: QueryBuilder<'a, Sqlite>,
}

pub enum Filter<T> {
	Equal(String, T),
	NotEqual(String, T),
}

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

	pub fn filter<T>(mut self, filter: Filter<T>) -> Self
	where
		T: sqlx::Type<Sqlite> + sqlx::Encode<'a, Sqlite> + Into<Value> + 'a,
	{
		self.builder.push(format!(" WHERE "));
		match filter {
			Filter::Equal(column, value) => {
				self.builder.push(column);
				self.builder.push(" = ");
				push_bind_value(&mut self.builder, value.into());
			}
			Filter::NotEqual(column, value) => {
				self.builder.push(column);
				self.builder.push(" != ");
				push_bind_value(&mut self.builder, value.into());
			}
		}

		self
	}

	pub fn insert<T>(mut self, rows: &Vec<T>) -> Self
	where
		T: TableLike,
	{
		if !self.builder.sql().is_empty() {
			panic!("insert can't be chained with previous queries");
		}

		let table_name = T::table_name();
		let column_names = T::columns();

		self.builder.push("INSERT INTO ");
		self.builder.push(&table_name);
		self.builder.push(" (");
		self.builder.push(column_names.join(", "));
		self.builder.push(")");
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

				push_bind_value(&mut self.builder, value);
			}

			self.builder.push(")");
		}

		self
	}

	pub fn update<T>(mut self, row: &T) -> Self
	where
		T: TableLike,
	{
		if !self.builder.sql().is_empty() {
			panic!("update can't be chained with previous queries");
		}

		let table_name = T::table_name();
		let column_names = T::columns();
		let primary_columns = T::primary_key_columns();

		self.builder.push("UPDATE ");
		self.builder.push(&table_name);
		self.builder.push(" SET ");

		let values = row.to_values();
		let update_columns: Vec<&String> = column_names
			.iter()
			.filter(|column| !primary_columns.contains(column))
			.collect();

		for (i, column) in update_columns.iter().enumerate() {
			if i > 0 {
				self.builder.push(", ");
			}

			self.builder.push(column);
			self.builder.push(" = ");

			let column_index = column_names.iter().position(|c| c == *column).unwrap();
			let value = &values[column_index];
			push_bind_value(&mut self.builder, value.clone());
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

fn push_bind_value(builder: &mut QueryBuilder<Sqlite>, value: Value) {
	match value {
		Value::Bool(v) => builder.push_bind(v),
		Value::String(v) => builder.push_bind(v),
		Value::Integer(v) => builder.push_bind(v),
		Value::Real(v) => builder.push_bind(v),
		Value::Blob(v) => builder.push_bind(v),
		Value::Null => builder.push_bind::<Option<i32>>(None),
	};
}
