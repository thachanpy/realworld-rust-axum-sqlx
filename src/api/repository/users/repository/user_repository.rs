use crate::api::repository::field_value::FieldValue;
use crate::api::repository::pagination::PaginationHelper;
use crate::api::repository::sorting::SortingHelper;
use crate::api::repository::users::constant::user_constant::{
  OAuth2Provider, UserRole, UserStatus,
};
use crate::api::repository::users::entity::user_entity::{UserEntity, Users};
use crate::api::repository::RepositoryImpl;
use crate::core::error::error::AppError;
use crate::core::request::pagination::Pagination;
use crate::core::request::sorting::Sorting;
use crate::db::db::DbPool;
use crate::utils::datetime_utils::TimeUtils;
use async_trait::async_trait;
use sea_query::{ColumnRef, Expr, Order, PostgresQueryBuilder, Query, SimpleExpr};
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
  async fn create(
    &self,
    email: String,
    password: Option<String>,
    name: Option<String>,
    auth_id: Option<String>,
    auth_provider: Option<OAuth2Provider>,
  ) -> Result<UserEntity, AppError>;
  async fn update(&self, id: Uuid, fields: Vec<(Users, FieldValue)>) -> Result<(), AppError>;
  async fn update_logged_in_at(&self, id: Uuid) -> Result<(), AppError>;
  async fn by_email(&self, email: String) -> Result<Option<UserEntity>, AppError>;
  async fn by_auth_provider(
    &self,
    auth_id: String,
    auth_provider: OAuth2Provider,
  ) -> Result<Option<UserEntity>, AppError>;
  async fn all(
    &self,
    pagination: Pagination,
    sorting: Sorting,
  ) -> Result<Vec<UserEntity>, AppError>;
  async fn me(&self, id: Uuid) -> Result<UserEntity, AppError>;
  async fn update_profile_url(&self, id: Uuid, profile_url: String) -> Result<(), AppError>;
  async fn update_role(&self, id: Uuid, role: UserRole) -> Result<(), AppError>;
  async fn update_status(&self, id: Uuid, status: UserStatus) -> Result<(), AppError>;
}

#[async_trait]
impl UserRepository for RepositoryImpl {
  async fn create(
    &self,
    email: String,
    password: Option<String>,
    name: Option<String>,
    auth_id: Option<String>,
    auth_provider: Option<OAuth2Provider>,
  ) -> Result<UserEntity, AppError> {
    let conn: &DbPool = &self.db_pool.primary();

    let query: String = Query::insert()
      .into_table(Users::Table)
      .columns([
        Users::Email,
        Users::Password,
        Users::Name,
        Users::Role,
        Users::Status,
        Users::AuthId,
        Users::AuthProvider,
      ])
      .values_panic([
        email.into(),
        password
          .as_deref()
          .map(Expr::value)
          .unwrap_or(Expr::value(None::<&str>)),
        name
          .as_deref()
          .map(Expr::value)
          .unwrap_or(Expr::value(None::<&str>)),
        UserRole::User.into(),
        UserStatus::Registered.into(),
        auth_id
          .as_deref()
          .map(Expr::value)
          .unwrap_or(Expr::value(None::<&str>)),
        auth_provider
          .map(|provider| Expr::value(provider.as_str()))
          .unwrap_or(Expr::value(None::<&str>)),
      ])
      .returning_col(ColumnRef::Asterisk)
      .to_string(PostgresQueryBuilder);
    let user: Result<UserEntity, sqlx::Error> = sqlx::query_as::<_, UserEntity>(&query)
      .fetch_one(conn)
      .await;
    Ok(user?)
  }

  async fn update(&self, id: Uuid, fields: Vec<(Users, FieldValue)>) -> Result<(), AppError> {
    let conn: &DbPool = &self.db_pool.primary();

    let values: Vec<(Users, SimpleExpr)> = fields
      .into_iter()
      .map(|(field, value)| (field, value.to_simple_expr()))
      .collect();

    let query: String = Query::update()
      .table(Users::Table)
      .and_where(Expr::col(Users::Id).eq(id.to_string()))
      .values(values)
      .to_string(PostgresQueryBuilder);

    sqlx::query(&query).execute(conn).await?;
    Ok(())
  }

  async fn update_logged_in_at(&self, id: Uuid) -> Result<(), AppError> {
    self
      .update(
        id,
        vec![(
          Users::LoggedInAt,
          FieldValue::DateTime(TimeUtils::utc_now()),
        )],
      )
      .await
  }

  async fn by_email(&self, email: String) -> Result<Option<UserEntity>, AppError> {
    let conn: &DbPool = &self.db_pool.replica();
    let query: String = Query::select()
      .from(Users::Table)
      .column(ColumnRef::Asterisk)
      .and_where(Expr::col(Users::Email).eq(email))
      .and_where(Expr::col(Users::DeletedAt).is_null())
      .and_where(Expr::col(Users::AuthId).is_null())
      .to_string(PostgresQueryBuilder);

    let user: Result<Option<UserEntity>, sqlx::Error> = sqlx::query_as::<_, UserEntity>(&query)
      .fetch_optional(conn)
      .await;
    Ok(user?)
  }

  async fn by_auth_provider(
    &self,
    auth_id: String,
    auth_provider: OAuth2Provider,
  ) -> Result<Option<UserEntity>, AppError> {
    let conn: &DbPool = &self.db_pool.replica();
    let query: String = Query::select()
      .from(Users::Table)
      .column(ColumnRef::Asterisk)
      .and_where(Expr::col(Users::AuthId).eq(auth_id))
      .and_where(Expr::col(Users::AuthProvider).eq(auth_provider.as_str()))
      .and_where(Expr::col(Users::DeletedAt).is_null())
      .to_string(PostgresQueryBuilder);

    let user: Result<Option<UserEntity>, sqlx::Error> = sqlx::query_as::<_, UserEntity>(&query)
      .fetch_optional(conn)
      .await;
    Ok(user?)
  }

  async fn all(
    &self,
    pagination: Pagination,
    sorting: Sorting,
  ) -> Result<Vec<UserEntity>, AppError> {
    let conn: &DbPool = &self.db_pool.replica();

    let pagination_helper: PaginationHelper =
      PaginationHelper::new(pagination.page, pagination.per_page);

    let order_by: Users = Users::map_order_by(&sorting.order_by.unwrap()).unwrap();
    let order_direction: Order =
      SortingHelper::map_order_direction(&sorting.order_direction.unwrap());

    let query: String = Query::select()
      .from(Users::Table)
      .column(ColumnRef::Asterisk)
      .and_where(Expr::col(Users::DeletedAt).is_null())
      .limit(pagination_helper.limit())
      .offset(pagination_helper.offset())
      .order_by(order_by, order_direction)
      .to_string(PostgresQueryBuilder);

    let users: Result<Vec<UserEntity>, sqlx::Error> = sqlx::query_as::<_, UserEntity>(&query)
      .fetch_all(conn)
      .await;
    Ok(users?)
  }

  async fn me(&self, id: Uuid) -> Result<UserEntity, AppError> {
    let conn: &DbPool = &self.db_pool.replica();
    let query: String = Query::select()
      .from(Users::Table)
      .column(ColumnRef::Asterisk)
      .and_where(Expr::col(Users::Id).eq(id.to_string()))
      .and_where(Expr::col(Users::DeletedAt).is_null())
      .to_string(PostgresQueryBuilder);

    let user: Result<UserEntity, sqlx::Error> = sqlx::query_as::<_, UserEntity>(&query)
      .fetch_one(conn)
      .await;
    Ok(user?)
  }

  async fn update_profile_url(&self, id: Uuid, profile_url: String) -> Result<(), AppError> {
    self
      .update(id, vec![(Users::ProfileUrl, FieldValue::Text(profile_url))])
      .await
  }

  async fn update_role(&self, id: Uuid, role: UserRole) -> Result<(), AppError> {
    self
      .update(
        id,
        vec![(Users::Role, FieldValue::Enum(role.as_str().to_string()))],
      )
      .await
  }

  async fn update_status(&self, id: Uuid, status: UserStatus) -> Result<(), AppError> {
    self
      .update(
        id,
        vec![(Users::Status, FieldValue::Enum(status.as_str().to_string()))],
      )
      .await
  }
}
