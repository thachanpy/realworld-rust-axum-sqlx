use crate::api::repository::refresh_tokens::entity::refresh_tokens_entity::{
  RefreshTokenEntity, RefreshTokens,
};
use crate::api::repository::RepositoryImpl;
use crate::core::error::error::AppError;
use crate::db::db::DbPool;
use async_trait::async_trait;
use sea_query::{ColumnRef, Expr, PostgresQueryBuilder, Query};
use uuid::Uuid;

#[async_trait]
pub trait RefreshTokenRepository: Send + Sync + 'static {
  async fn by_id(&self, id: Uuid) -> Result<Option<RefreshTokenEntity>, AppError>;
  async fn create(&self, user_id: Uuid) -> Result<RefreshTokenEntity, AppError>;
  async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

#[async_trait]
impl RefreshTokenRepository for RepositoryImpl {
  async fn by_id(&self, id: Uuid) -> Result<Option<RefreshTokenEntity>, AppError> {
    let conn: &DbPool = &self.db_pool.replica();

    let query: String = Query::select()
      .from(RefreshTokens::Table)
      .column(ColumnRef::Asterisk)
      .and_where(Expr::col(RefreshTokens::Id).eq(id.to_string()))
      .to_string(PostgresQueryBuilder);

    let user: Result<Option<RefreshTokenEntity>, sqlx::Error> =
      sqlx::query_as::<_, RefreshTokenEntity>(&query)
        .fetch_optional(conn)
        .await;
    Ok(user?)
  }

  async fn create(&self, user_id: Uuid) -> Result<RefreshTokenEntity, AppError> {
    let conn: &DbPool = &self.db_pool.primary();
    let query: String = Query::insert()
      .into_table(RefreshTokens::Table)
      .columns([RefreshTokens::UserId])
      .values_panic([user_id.to_string().into()])
      .returning_col(ColumnRef::Asterisk)
      .to_string(PostgresQueryBuilder);
    let refresh_token: Result<RefreshTokenEntity, sqlx::Error> =
      sqlx::query_as::<_, RefreshTokenEntity>(&query)
        .fetch_one(conn)
        .await;
    Ok(refresh_token?)
  }

  async fn delete(&self, id: Uuid) -> Result<(), AppError> {
    let conn: &DbPool = &self.db_pool.primary();
    let query: String = Query::delete()
      .from_table(RefreshTokens::Table)
      .and_where(Expr::col(RefreshTokens::Id).eq(id.to_string()))
      .to_string(PostgresQueryBuilder);
    sqlx::query_as::<_, RefreshTokenEntity>(&query)
      .fetch_optional(conn)
      .await?;
    Ok(())
  }
}
