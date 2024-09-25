use crate::api::client::auth::response::auth_response::SignInResponse;
use crate::api::manager::auth::auth_manager::AuthManagerImpl;

pub trait AuthResponseConverter {
  fn auth_token_response_converter(
    access_token: String,
    refresh_token: Option<String>,
  ) -> SignInResponse;
}

impl AuthResponseConverter for AuthManagerImpl {
  fn auth_token_response_converter(
    access_token: String,
    refresh_token: Option<String>,
  ) -> SignInResponse {
    SignInResponse {
      access_token,
      refresh_token,
    }
  }
}
