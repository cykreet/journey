use sqlx::{sqlite::SqliteQueryResult, Error, Pool, Sqlite};

use crate::entities::TableLike;

pub struct SqlQuery<'a> {
	pool: Option<&'a Pool<Sqlite>>,
}

impl<'a> SqlQuery<'a> {
	pub fn new() -> Self {
		return Self { pool: None };
	}

	pub fn pool(&mut self, pool: &'a Pool<Sqlite>) -> &Self {
		self.pool = Some(pool);
		return self;
	}

	// pub fn select(&mut self, columns: &str) -> &Self {}

	pub async fn insert_into<T>(&self, rows: &Vec<T>) -> Result<SqliteQueryResult, Error>
	where
		T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + TableLike,
	{
		if self.pool.is_none() {
			panic!("pool not set");
		}

		let table_name = T::table_name();
		let column_names = T::column_names();
		let mut query = format!(
			"INSERT INTO {} ({}) VALUES",
			table_name,
			column_names.join(", ")
		);
		let values_per_row = column_names.len();
		let placeholder_row = format!(
			"({})",
			(0..values_per_row)
				.map(|_| "?")
				.collect::<Vec<_>>()
				.join(", ")
		);

		let placeholders = Vec::new();
		for _ in 0..rows.len() {
			placeholders.push(placeholder_row.clone());
		}

		query.push_str(&placeholders.join(", "));

		let pool = self.pool.unwrap();
		let mut db_query = sqlx::query(&query);
		for row in rows.iter() {
			let values = row.to_values();
			for value in values {
				db_query = db_query.bind(value); // todo: broken
			}
		}

		db_query.execute(pool).await
	}
}
