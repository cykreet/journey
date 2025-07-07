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
		self
	}

	pub async fn select<T>(&self) -> Result<Vec<T>, Error>
	where
		T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + TableLike + Send + Unpin,
	{
		if self.pool.is_none() {
			panic!("pool not set");
		}

		let pool = self.pool.unwrap();
		let query_str = format!("SELECT * FROM {}", T::table_name());
		let rows = sqlx::query_as(&query_str).fetch_all(pool).await?;

		Ok(rows)
	}

	pub async fn select_where<T>(
		&self,
		where_clause: &str,
		args: &Vec<String>,
	) -> Result<Vec<T>, Error>
	where
		T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + TableLike + Send + Unpin,
	{
		if self.pool.is_none() {
			panic!("pool not set");
		}

		let pool = self.pool.unwrap();
		let table_name = T::table_name();
		let query_str = format!("SELECT * FROM {} WHERE {}", table_name, where_clause);
		let mut query = sqlx::query_as::<_, T>(&query_str);
		for arg in args {
			query = query.bind(arg);
		}

		query.fetch_all(pool).await
	}

	pub async fn insert_into<T>(&self, rows: &Vec<T>) -> Result<SqliteQueryResult, Error>
	where
		T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + TableLike,
	{
		if self.pool.is_none() {
			panic!("pool not set");
		}

		let table_name = T::table_name();
		let column_names = T::column_names();
		let values_per_row = column_names.len();

		let placeholder_row = format!(
			"({})",
			(0..values_per_row)
				.map(|_| "?")
				.collect::<Vec<_>>()
				.join(", ")
		);

		let placeholder_rows: Vec<String> = (0..rows.len()).map(|_| placeholder_row.clone()).collect();

		// todo: doubt all queries would want OR REPLACE here
		let query_str = format!(
			"INSERT OR REPLACE INTO {} ({}) VALUES {}",
			table_name,
			column_names.join(", "),
			placeholder_rows.join(", ")
		);

		let pool = self.pool.unwrap();
		let mut query = sqlx::query(&query_str);
		for row in rows {
			query = row.bind_to_query(query);
		}

		query.execute(pool).await
	}
}
