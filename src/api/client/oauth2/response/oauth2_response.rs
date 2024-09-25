use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OAuth2UserInfo {
  pub(crate) sub: String,
  pub(crate) email: String,
  pub(crate) name: Option<String>,
  pub(crate) picture: Option<String>,
}
