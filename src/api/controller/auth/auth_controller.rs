use crate::api::client::auth::request::auth_request::{
  AuthSignInRequest, AuthSignUpRequest, OAuth2SignInRequest,
};
use crate::api::client::auth::response::auth_response::SignInResponse;
use crate::api::manager::auth::auth_manager::AuthManager;
use crate::api::repository::users::constant::user_constant::OAuth2Provider;
use crate::api::state::auth::auth_state::AuthState;
use crate::core::error::error::AppError;
use crate::core::response::constant::BaseMessage;
use crate::core::response::response::{CreatedResponse, SuccessResponse};
use crate::core::security::authentication::{AccessTokenAuth, RefreshTokenAuth};
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{response::IntoResponse, Json, Router};
use std::sync::Arc;

pub struct AuthController;

impl AuthController {
  pub(crate) fn configure(state: Arc<AuthState>) -> Router {
    Router::new()
      .nest(
        "/auth",
        Router::new()
          .route("/sign-up", post(Self::sign_up))
          .route("/sign-in", post(Self::sign_in))
          .route(
            "/sign-in/:provider",
            get(Self::sign_in_oauth2_get_redirect_uri),
          )
          .route("/sign-in/:provider", post(Self::sign_in_oauth2))
          .route("/sign-out", post(Self::sign_out))
          .route("/refresh-token", post(Self::refresh_token)),
      )
      .with_state(state.clone())
  }
}

impl AuthController {
  async fn sign_up(
    State(state): State<Arc<AuthState>>,
    Json(payload): Json<AuthSignUpRequest>,
  ) -> Result<impl IntoResponse, AppError> {
    state.manager.sign_up(payload).await?;
    Ok(
      CreatedResponse {
        message: BaseMessage::Created.into(),
      }
      .into_response(),
    )
  }

  async fn sign_in(
    State(state): State<Arc<AuthState>>,
    Json(payload): Json<AuthSignInRequest>,
  ) -> Result<impl IntoResponse, AppError> {
    let token: SignInResponse = state.manager.sign_in(payload).await?;
    Ok(SuccessResponse { data: token.into() }.into_response())
  }

  async fn sign_in_oauth2_get_redirect_uri(
    State(state): State<Arc<AuthState>>,
    provider: Path<OAuth2Provider>,
  ) -> Result<impl IntoResponse, AppError> {
    let redirect_url: String = state
      .manager
      .sign_in_oauth2_get_redirect_uri(provider.clone())
      .await?;
    Ok(
      SuccessResponse {
        data: redirect_url.into(),
      }
      .into_response(),
    )
  }

  async fn sign_in_oauth2(
    State(state): State<Arc<AuthState>>,
    provider: Path<OAuth2Provider>,
    Json(payload): Json<OAuth2SignInRequest>,
  ) -> Result<impl IntoResponse, AppError> {
    let token: SignInResponse = state
      .manager
      .sign_in_oauth2(provider.clone(), payload)
      .await?;
    Ok(SuccessResponse { data: token.into() }.into_response())
  }

  async fn sign_out(
    State(state): State<Arc<AuthState>>,
    AccessTokenAuth(token_data): AccessTokenAuth,
  ) -> Result<impl IntoResponse, AppError> {
    state.manager.sign_out(token_data.jti).await?;
    Ok(
      SuccessResponse {
        data: BaseMessage::Success.into(),
      }
      .into_response(),
    )
  }

  async fn refresh_token(
    State(state): State<Arc<AuthState>>,
    RefreshTokenAuth(token_data): RefreshTokenAuth,
  ) -> Result<impl IntoResponse, AppError> {
    let token: SignInResponse = state
      .manager
      .refresh_token(token_data.jti, token_data.sub, token_data.role)
      .await?;
    Ok(SuccessResponse { data: token.into() }.into_response())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::api::client::auth::auth_client::AuthClient;
  use crate::api::client::auth::response::auth_response::SignInResponse;
  use crate::api::test::test_context::{
    AuthSetup, TestClient, USER_ACCESS_TOKEN, USER_DEFAULT_EMAIL, USER_REFRESH_TOKEN,
  };
  use axum::body::Body;
  use axum::http::{Request, StatusCode};
  use axum::response::Response;
  use faker_rand::fr_fr::internet::Email;
  use faker_rand::fr_fr::names::FullName;
  use serde_json::{json, Value};
  use std::sync::MutexGuard;

  #[tokio::test]
  async fn test_auth_sign_up_success() {
    let (app, _, _) = AuthSetup::init().await;

    let payload = AuthSignUpRequest {
      email: rand::random::<Email>().to_string(),
      password: rand::random::<FullName>().to_string(),
      name: Option::from(rand::random::<FullName>().to_string()),
    };

    let request: Request<Body> = AuthClient::sign_up(&payload);

    let test_client: TestClient = TestClient::new(app);
    let response: Response = test_client.get_response(request).await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body: Value = test_client.get_body(response).await;

    assert_eq!(body, json!({ "message": "created" }));
  }

  #[tokio::test]
  async fn test_auth_sign_up_existing_email() {
    let (app, state, _) = AuthSetup::init().await;

    let email: String = rand::random::<FullName>().to_string();
    let payload = AuthSignUpRequest {
      email,
      password: rand::random::<FullName>().to_string(),
      name: Option::from(rand::random::<FullName>().to_string()),
    };

    state.manager.sign_up(payload.clone()).await.unwrap();

    let request: Request<Body> = AuthClient::sign_up(&payload);

    let test_client: TestClient = TestClient::new(app);
    let response: Response = test_client.get_response(request).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body: Value = test_client.get_body(response).await;

    assert_eq!(
      body,
      json!({
          "error": "existing email",
          "code": 409
      })
    );
  }

  #[tokio::test]
  async fn test_auth_sign_in_success() {
    AuthSetup::init().await;

    let access_token: MutexGuard<Option<String>> = USER_ACCESS_TOKEN.lock().unwrap();
    let refresh_token: MutexGuard<Option<String>> = USER_REFRESH_TOKEN.lock().unwrap();

    access_token
      .as_ref()
      .expect("Access Token should be initialized");
    refresh_token
      .as_ref()
      .expect("Refresh Token should be initialized");
  }

  #[tokio::test]
  async fn test_auth_sign_in_user_not_found() {
    let (app, _, _) = AuthSetup::init().await;

    let email: String = rand::random::<Email>().to_string();
    let password: String = rand::random::<FullName>().to_string();

    let payload = AuthSignInRequest { email, password };

    let request: Request<Body> = AuthClient::sign_in(&payload);

    let test_client: TestClient = TestClient::new(app);
    let response: Response = test_client.get_response(request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: Value = test_client.get_body(response).await;

    assert_eq!(
      body,
      json!({
          "error": "user not found",
          "code": 404
      })
    );
  }

  #[tokio::test]
  async fn test_auth_sign_in_invalid_password() {
    let (app, _, _) = AuthSetup::init().await;

    AuthSetup::initialize_user_tokens().await;

    let payload = AuthSignInRequest {
      email: USER_DEFAULT_EMAIL.clone(),
      password: rand::random::<FullName>().to_string(),
    };

    let request: Request<Body> = AuthClient::sign_in(&payload);

    let test_client: TestClient = TestClient::new(app);
    let response: Response = test_client.get_response(request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body: Value = test_client.get_body(response).await;

    assert_eq!(
      body,
      json!({
          "error": "incorrect password",
          "code": 401
      })
    );
  }

  #[tokio::test]
  async fn test_auth_get_oauth_redirect_url_success() {
    let (app, _, _) = AuthSetup::init().await;

    let provider: OAuth2Provider = OAuth2Provider::Google;

    let request: Request<Body> = AuthClient::sign_in_oauth2_get_redirect_uri(&provider);

    let test_client: TestClient = TestClient::new(app);
    let response: Response = test_client.get_response(request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let _: Value = test_client.get_body(response).await;
  }

  #[tokio::test]
  async fn test_auth_sign_out_unauthorized() {
    let (app, _, _) = AuthSetup::init().await;

    let test_client: TestClient = TestClient::new(app);
    test_client
      .call_unauthorized_request(
        AuthClient::sign_out,
        "fake_token".to_string(),
        json!({
            "error": "invalid jwt token",
            "code": 401
        }),
      )
      .await;
  }

  #[tokio::test]
  async fn test_auth_refresh_token_success() {
    let (app, _, _) = AuthSetup::init().await;
    AuthSetup::initialize_user_tokens().await;

    let refresh_token: String = AuthSetup::get_user_refresh_token();
    let request: Request<Body> = AuthClient::refresh_token(refresh_token);

    let test_client: TestClient = TestClient::new(app);
    let response: Response = test_client.get_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = test_client.get_body(response).await;

    if let Some(data) = body.get("data") {
      let data_value: SignInResponse =
        serde_json::from_value(data.clone()).expect("Failed to deserialize data");
      assert!(
        !data_value.access_token.is_empty(),
        "'access_token' field should not be empty"
      );
      assert!(
        data_value.refresh_token.is_none(),
        "'refresh_token' field should none"
      );
    } else {
      panic!("Expected 'data' field not found in response body");
    }
  }

  #[tokio::test]
  async fn test_auth_refresh_unauthorized() {
    let (app, _, _) = AuthSetup::init().await;

    let test_client: TestClient = TestClient::new(app);
    test_client
      .call_unauthorized_request(
        AuthClient::refresh_token,
        "fake_token".to_string(),
        json!({
            "error": "invalid jwt token",
            "code": 401
        }),
      )
      .await;
  }

  #[tokio::test]
  async fn test_auth_refresh_use_access_token() {
    let (app, _, _) = AuthSetup::init().await;

    let refresh_token: String = AuthSetup::get_user_access_token();

    let test_client: TestClient = TestClient::new(app);
    test_client
      .call_unauthorized_request(
        AuthClient::refresh_token,
        refresh_token,
        json!({
            "error": "invalid jwt token",
            "code": 401
        }),
      )
      .await;
  }
}
