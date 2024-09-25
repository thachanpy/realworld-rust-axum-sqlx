use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthSignUpRequest {
  pub email: String,
  pub password: String,
  pub name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthSignInRequest {
  pub email: String,
  pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OAuth2SignInRequest {
  pub code: String,
}
