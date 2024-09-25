use crate::api::client::users::request::user_request::UserUpdateRoleRequest;
use crate::api::manager::users::user_manager::UserManager;
use crate::api::state::users::user_state::UserState;
use crate::core::error::error::AppError;
use crate::core::request::pagination::Pagination;
use crate::core::request::sorting::Sorting;
use crate::core::response::constant::BaseMessage;
use crate::core::response::response::SuccessResponse;
use crate::core::security::authentication::AccessTokenAuth;
use crate::core::security::authorization::Authorization;
use axum::extract::{Multipart, State};
use axum::middleware::from_fn;
use axum::routing::{get, post, put};
use axum::{response::IntoResponse, Json, Router};
use std::sync::Arc;

pub struct UserController;

impl UserController {
  pub(crate) fn configure(state: Arc<UserState>) -> Router {
    Router::new()
      .nest(
        "/users",
        Router::new()
          .route("/", get(Self::all).layer(from_fn(Authorization::admin)))
          .route("/me", get(Self::me))
          .route("/profile", post(Self::upload_profile_url))
          .route(
            "/role",
            put(Self::update_role).layer(from_fn(Authorization::admin)),
          ),
      )
      .with_state(state.clone())
  }
}

impl UserController {
  async fn all(
    State(state): State<Arc<UserState>>,
    AccessTokenAuth(_): AccessTokenAuth,
    pagination: Pagination,
    sorting: Sorting,
  ) -> Result<impl IntoResponse, AppError> {
    Ok(SuccessResponse {
      data: state.manager.all(pagination, sorting).await?.into(),
    })
  }

  async fn me(
    State(state): State<Arc<UserState>>,
    AccessTokenAuth(token_data): AccessTokenAuth,
  ) -> Result<impl IntoResponse, AppError> {
    Ok(SuccessResponse {
      data: state.manager.me(token_data.sub).await?.into(),
    })
  }

  async fn upload_profile_url(
    State(state): State<Arc<UserState>>,
    AccessTokenAuth(token_data): AccessTokenAuth,
    file: Multipart,
  ) -> Result<impl IntoResponse, AppError> {
    Ok(
      SuccessResponse {
        data: state
          .manager
          .upload_profile_url(token_data.sub, file)
          .await?,
      }
      .into_response(),
    )
  }

  async fn update_role(
    State(state): State<Arc<UserState>>,
    AccessTokenAuth(_): AccessTokenAuth,
    Json(payload): Json<UserUpdateRoleRequest>,
  ) -> Result<impl IntoResponse, AppError> {
    state.manager.update_role(payload).await?;
    Ok(SuccessResponse {
      data: BaseMessage::Updated.into(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::api::client::users::response::user_response::UserResponse;
  use crate::api::client::users::user_client::UserClient;
  use crate::api::repository::users::constant::user_constant::UserRole;
  use crate::api::test::test_context::{
    AuthSetup, TestClient, USER_DEFAULT_EMAIL, USER_DEFAULT_USER_ID,
  };
  use crate::db::db::PostgresDatabase;
  use crate::launcher::APP_CONFIG;
  use crate::resources::config::AppConfig;
  use axum::body::Body;
  use axum::http::Request;
  use axum::response::Response;
  use http::StatusCode;
  use serde_json::{json, Value};
  use std::sync::Arc;
  use uuid::Uuid;

  async fn init_setup() -> (Router, Arc<UserState>) {
    let config: AppConfig = APP_CONFIG.clone();
    let db_pool: PostgresDatabase = PostgresDatabase::connect(config.postgres).await.unwrap();
    let state: Arc<UserState> = Arc::new(UserState::new(&db_pool, &config.aws).await);
    let app: Router = UserController::configure(state.clone());
    (app, state)
  }

  #[tokio::test]
  async fn test_user_all_success() {
    let (app, _) = init_setup().await;
    AuthSetup::initialize_admin_tokens().await;

    let access_token: String = AuthSetup::get_admin_access_token();
    let request: Request<Body> = UserClient::all(access_token);

    let test_client: TestClient = TestClient::new(app);
    let response: Response = test_client.get_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = test_client.get_body(response).await;

    if let Some(data) = body.get("data") {
      assert!(data.is_array());
      let data_array: Vec<UserResponse> =
        serde_json::from_value(data.clone()).expect("Failed to deserialize data");
      assert!(!data_array.is_empty());
    } else {
      panic!("Expected 'data' field not found in response body");
    }
  }

  #[tokio::test]
  async fn test_user_all_unauthorized() {
    let (app, _) = init_setup().await;

    let test_client: TestClient = TestClient::new(app);
    test_client
      .call_unauthorized_request(
        UserClient::all,
        "fake_token".to_string(),
        json!({
            "error": "invalid jwt token",
            "code": 401
        }),
      )
      .await;
  }

  #[tokio::test]
  async fn test_user_all_permission_denied() {
    let (app, _) = init_setup().await;
    AuthSetup::initialize_user_tokens().await;

    let access_token: String = AuthSetup::get_user_access_token();

    let test_client: TestClient = TestClient::new(app);
    test_client
      .call_permission_denied_request(
        UserClient::all,
        access_token,
        json!({
            "error": "permission denied",
            "code": 403
        }),
      )
      .await;
  }

  #[tokio::test]
  async fn test_user_me_success() {
    let (app, _) = init_setup().await;
    AuthSetup::initialize_user_tokens().await;

    let access_token: String = AuthSetup::get_user_access_token();
    let request: Request<Body> = UserClient::me(access_token);

    let test_client: TestClient = TestClient::new(app);
    let response: Response = test_client.get_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = test_client.get_body(response).await;

    if let Some(data) = body.get("data") {
      let data_value: UserResponse =
        serde_json::from_value(data.clone()).expect("Failed to deserialize data");
      assert!(!data_value.id.is_nil(), "'id' field should not be empty");

      assert_eq!(data_value.email, USER_DEFAULT_EMAIL.clone());
    } else {
      panic!("Expected 'data' field not found in response body");
    }
  }

  #[tokio::test]
  async fn test_user_me_unauthorized() {
    let (app, _) = init_setup().await;

    let test_client: TestClient = TestClient::new(app);
    test_client
      .call_unauthorized_request(
        UserClient::me,
        "fake_token".to_string(),
        json!({
            "error": "invalid jwt token",
            "code": 401
        }),
      )
      .await;
  }

  #[tokio::test]
  async fn test_user_update_profile_url_success() {
    let (app, _) = init_setup().await;
    AuthSetup::initialize_user_tokens().await;

    let access_token: String = AuthSetup::get_user_access_token();

    let boundary = "boundary".to_string();
    let body = format!(
      "--{}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"profile.png\"\r\n\
        Content-Type: image/png\r\n\r\n\
        {}\
        \r\n--{}--",
      boundary, "example content", boundary
    );

    let request: Request<Body> = UserClient::update_profile_url(body, boundary, access_token);

    let test_client: TestClient = TestClient::new(app);
    let response: Response = test_client.get_response(request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = test_client.get_body(response).await;

    if let Some(data) = body.get("data") {
      assert!(data.is_string());
    } else {
      panic!("Expected 'data' field not found in response body");
    }
  }

  #[tokio::test]
  async fn test_user_update_profile_url_unauthorized() {
    let (app, _) = init_setup().await;

    let test_client: TestClient = TestClient::new(app);

    let adapted_endpoint = |_: String| {
      UserClient::update_profile_url(
        "body".to_string(),
        "boundary".to_string(),
        "fake_token".to_string(),
      )
    };

    test_client
      .call_unauthorized_request(
        adapted_endpoint,
        "fake_token".to_string(),
        json!({
            "error": "invalid jwt token",
            "code": 401
        }),
      )
      .await;
  }

  #[tokio::test]
  async fn test_user_update_role_success() {
    let (app, _) = init_setup().await;
    AuthSetup::initialize_admin_tokens().await;
    AuthSetup::initialize_user_tokens().await;

    let payload = UserUpdateRoleRequest {
      user_id: AuthSetup::get_uuid_from_guard(USER_DEFAULT_USER_ID.lock().unwrap()).unwrap(),
      role: UserRole::User,
    };

    let access_token: String = AuthSetup::get_admin_access_token();

    let request: Request<Body> = UserClient::update_role(&payload, access_token);

    let test_client: TestClient = TestClient::new(app);
    let response: Response = test_client.get_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);
  }

  #[tokio::test]
  async fn test_user_update_role_unauthorized() {
    let (app, _) = init_setup().await;

    let payload = UserUpdateRoleRequest {
      user_id: Uuid::new_v4(),
      role: UserRole::User,
    };

    let test_client: TestClient = TestClient::new(app);
    test_client
      .call_unauthorized_request(
        |_| UserClient::update_role(&payload, "fake_token".to_string()),
        "fake_token".to_string(),
        json!({
            "error": "invalid jwt token",
            "code": 401
        }),
      )
      .await;
  }

  #[tokio::test]
  async fn test_user_update_role_permission_denied() {
    let (app, _) = init_setup().await;
    AuthSetup::initialize_user_tokens().await;

    let access_token: String = AuthSetup::get_user_access_token();

    let payload = UserUpdateRoleRequest {
      user_id: Uuid::new_v4(),
      role: UserRole::User,
    };

    let test_client: TestClient = TestClient::new(app);
    test_client
      .call_permission_denied_request(
        |_| UserClient::update_role(&payload, access_token.clone()),
        access_token.clone(),
        json!({
            "error": "permission denied",
            "code": 403
        }),
      )
      .await;
  }
}
