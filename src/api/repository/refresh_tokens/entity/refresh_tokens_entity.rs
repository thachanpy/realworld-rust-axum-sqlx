use chrono::{DateTime, Utc};
use sea_query::Iden;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Eq, Hash, Iden, PartialEq)]
pub enum RefreshTokens {
  Table,
  Id,
  UserId,
  CreatedAt,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct RefreshTokenEntity {
  pub id: Uuid,
  pub user_id: Uuid,
  pub created_at: Option<DateTime<Utc>>,
}
