use crate::db::db::PostgresDatabase;

mod field_value;
mod pagination;
pub(crate) mod refresh_tokens;
mod sorting;
pub(crate) mod users;

#[derive(Clone)]
pub struct RepositoryImpl {
  pub(crate) db_pool: PostgresDatabase,
}

impl RepositoryImpl {
  pub fn new(db_pool: PostgresDatabase) -> Self {
    Self { db_pool }
  }
}
