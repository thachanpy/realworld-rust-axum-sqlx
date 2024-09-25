use sea_query::Value;
use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Serialize, Deserialize, Clone, Debug, Type, PartialEq, Copy)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum UserRole {
  #[serde(rename = "user")]
  User,
  #[serde(rename = "admin")]
  Admin,
}

impl UserRole {
  pub fn as_str(&self) -> &str {
    match self {
      UserRole::User => "user",
      UserRole::Admin => "admin",
    }
  }
}

impl From<UserRole> for Value {
  fn from(role: UserRole) -> Self {
    let value: &str = match role {
      UserRole::User => "user",
      UserRole::Admin => "admin",
    };
    Value::from(value.to_string())
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[sqlx(type_name = "user_status", rename_all = "snake_case")]
pub enum UserStatus {
  #[serde(rename = "registered")]
  Registered,
  #[serde(rename = "verified")]
  Verified,
}

impl UserStatus {
  pub fn as_str(&self) -> &str {
    match self {
      UserStatus::Registered => "registered",
      UserStatus::Verified => "verified",
    }
  }
}

impl From<UserStatus> for Value {
  fn from(status: UserStatus) -> Self {
    let value: &str = match status {
      UserStatus::Registered => "registered",
      UserStatus::Verified => "verified",
    };
    Value::from(value.to_string())
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, Type, PartialEq, Copy)]
#[sqlx(type_name = "oauth2_provider", rename_all = "snake_case")]
pub enum OAuth2Provider {
  #[serde(rename = "google")]
  Google,
}

impl OAuth2Provider {
  pub fn as_str(&self) -> &str {
    match self {
      OAuth2Provider::Google => "google",
    }
  }
}

impl From<OAuth2Provider> for Value {
  fn from(provider: OAuth2Provider) -> Self {
    let value: &str = match provider {
      OAuth2Provider::Google => "google",
    };
    Value::from(value.to_string())
  }
}
