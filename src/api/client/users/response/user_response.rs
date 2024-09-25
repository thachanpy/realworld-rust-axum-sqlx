use crate::api::repository::users::constant::user_constant::UserRole;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
  pub id: Uuid,
  pub email: String,
  pub name: Option<String>,
  pub role: UserRole,
  pub profile_url: Option<String>,
  pub logged_in_at: Option<DateTime<Utc>>,
  pub created_at: Option<DateTime<Utc>>,
  pub updated_at: Option<DateTime<Utc>>,
}
