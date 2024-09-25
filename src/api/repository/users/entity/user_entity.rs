use crate::api::repository::users::constant::user_constant::{
  OAuth2Provider, UserRole, UserStatus,
};
use chrono::{DateTime, Utc};
use sea_query::Iden;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Eq, Hash, Iden, PartialEq)]
pub enum Users {
  Table,
  Id,
  Email,
  Password,
  Name,
  Role,
  Status,
  AuthId,
  AuthProvider,
  ProfileUrl,
  LoggedInAt,
  CreatedAt,
  UpdatedAt,
  DeletedAt,
}

impl Users {
  pub fn map_order_by(order_by: &str) -> Option<Users> {
    match order_by.to_lowercase().as_str() {
      "name" => Option::from(Users::Name),
      "updated_at" => Option::from(Users::UpdatedAt),
      _ => Option::from(Users::CreatedAt),
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct UserEntity {
  pub id: Uuid,
  pub email: String,
  pub password: Option<String>,
  pub name: Option<String>,
  pub profile_url: Option<String>,
  pub role: UserRole,
  pub status: UserStatus,
  pub auth_id: Option<String>,
  pub auth_provider: Option<OAuth2Provider>,
  pub logged_in_at: Option<DateTime<Utc>>,
  pub created_at: Option<DateTime<Utc>>,
  pub updated_at: Option<DateTime<Utc>>,
  pub deleted_at: Option<DateTime<Utc>>,
}
