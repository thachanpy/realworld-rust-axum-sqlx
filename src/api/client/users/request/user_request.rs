use crate::api::repository::users::constant::user_constant::{UserRole, UserStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct UserUpdateRoleRequest {
  pub user_id: Uuid,
  pub role: UserRole,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserUpdateStatusRequest {
  pub user_id: Uuid,
  pub status: UserStatus,
}
