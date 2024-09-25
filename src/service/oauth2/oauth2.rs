use crate::api::client::auth::request::auth_request::OAuth2SignInRequest;
use crate::api::client::oauth2::response::oauth2_response::OAuth2UserInfo;
use crate::core::error::error::AppError;
use crate::resources::config::OAuth2Config;
use async_trait::async_trait;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
  AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse,
  TokenUrl,
};
use reqwest::Client;
use tracing::error;

#[async_trait]
pub trait OAuth2Service: Send + Sync + 'static {
  async fn get_redirect_url(&self) -> Result<String, AppError>;
  async fn sign_in(&self, payload: OAuth2SignInRequest) -> Result<OAuth2UserInfo, AppError>;
}

#[derive(Clone)]
pub struct OAuth2ServiceImpl {
  client: BasicClient,
  scopes: Vec<String>,
  user_info_url: String,
}

impl OAuth2ServiceImpl {
  pub async fn new(config: OAuth2Config) -> Self {
    let client: BasicClient = BasicClient::new(
      ClientId::new(config.client_id),
      Some(ClientSecret::new(config.client_secret)),
      AuthUrl::new(config.auth_url).unwrap(),
      Some(TokenUrl::new(config.token_url).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(config.redirect_url).unwrap());

    OAuth2ServiceImpl {
      client,
      scopes: config.scopes,
      user_info_url: config.user_info_url,
    }
  }
}

#[async_trait]
impl OAuth2Service for OAuth2ServiceImpl {
  async fn get_redirect_url(&self) -> Result<String, AppError> {
    let (auth_url, _) = self
      .scopes
      .clone()
      .into_iter()
      .fold(
        self.client.authorize_url(CsrfToken::new_random),
        |acc, scope| acc.add_scope(Scope::new(scope)),
      )
      .url();
    Ok(auth_url.to_string())
  }

  async fn sign_in(&self, payload: OAuth2SignInRequest) -> Result<OAuth2UserInfo, AppError> {
    let token: String = match self
      .client
      .exchange_code(AuthorizationCode::new(payload.code))
      .request_async(async_http_client)
      .await
    {
      Ok(resp) => resp.access_token().secret().to_string(),
      Err(e) => {
        error!("Error during token exchange: {:?}", e);
        return Err(AppError::SomethingWentWrong);
      }
    };

    let client: Client = Client::new();
    let user_info: OAuth2UserInfo = match client
      .get(self.user_info_url.clone())
      .bearer_auth(&token)
      .send()
      .await
    {
      Ok(resp) if resp.status().is_success() => resp.json().await.map_err(|e| {
        error!("Error parsing user info: {:?}", e);
        return AppError::SomethingWentWrong;
      })?,
      Ok(resp) => {
        error!("Failed to retrieve user info, status: {}", resp.status());
        return Err(AppError::SomethingWentWrong);
      }
      Err(e) => {
        error!("Error making request to user info endpoint: {:?}", e);
        return Err(AppError::SomethingWentWrong);
      }
    };
    Ok(user_info)
  }
}
