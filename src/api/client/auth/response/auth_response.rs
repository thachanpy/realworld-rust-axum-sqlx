use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignInResponse {
  pub access_token: String,
  pub refresh_token: Option<String>,
}
