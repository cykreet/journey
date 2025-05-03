use convert_case::{Case, Casing};
use sqlx::{sqlite::SqliteQueryResult, Error, Pool, Sqlite};

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

	// pub async fn insert_into<T: sqlx::FromRow + 'static>(
	// 	&self,
	// 	rows: Vec<T>,
	// ) -> Result<SqliteQueryResult, Error> {
	// 	if self.pool.is_none() {
	// 		panic!("pool not set");
	// 	}

	// 	let type_name = std::any::type_name::<T>().split("::").last().unwrap();
	// 	let table_name = type_name.to_case(Case::Snake);
	// 	return sqlx::query(&query).execute(*pool).await;
	// }
}
