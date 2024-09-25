use crate::api::client::auth::auth_client::AuthClient;
use crate::api::client::auth::request::auth_request::{AuthSignInRequest, AuthSignUpRequest};
use crate::api::client::users::request::user_request::UserUpdateRoleRequest;
use crate::api::controller::auth::auth_controller::AuthController;
use crate::api::manager::auth::auth_manager::AuthManager;
use crate::api::manager::users::user_manager::UserManager;
use crate::api::repository::users::constant::user_constant::UserRole;
use crate::api::repository::users::entity::user_entity::UserEntity;
use crate::api::state::auth::auth_state::AuthState;
use crate::api::state::users::user_state::UserState;
use crate::core::security::jwt_manager::JwtManager;
use crate::db::db::PostgresDatabase;
use crate::launcher::APP_CONFIG;
use crate::resources::config::AppConfig;
use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::response::Response;
use axum::Router;
use faker_rand::en_us::internet::Email;
use faker_rand::fr_fr::names::FullName;
use http::StatusCode;
use http_body_util::BodyExt;
use lazy_static::lazy_static;
use serde_json::{Map, Value};
use std::sync::{Arc, Mutex, MutexGuard};
use tower::ServiceExt;
use uuid::Uuid;

lazy_static! {
  pub static ref USER_DEFAULT_EMAIL: String = rand::random::<Email>().to_string();
  pub static ref USER_DEFAULT_PASSWORD: String = rand::random::<FullName>().to_string();
  pub static ref USER_DEFAULT_USER_ID: Mutex<Option<Uuid>> = Mutex::new(None);
  pub static ref USER_ACCESS_TOKEN: Mutex<Option<String>> = Mutex::new(None);
  pub static ref USER_REFRESH_TOKEN: Mutex<Option<String>> = Mutex::new(None);
  pub static ref ADMIN_DEFAULT_EMAIL: String = rand::random::<Email>().to_string();
  pub static ref ADMIN_DEFAULT_PASSWORD: String = rand::random::<FullName>().to_string();
  pub static ref ADMIN_DEFAULT_USER_ID: Mutex<Option<Uuid>> = Mutex::new(None);
  pub static ref ADMIN_ACCESS_TOKEN: Mutex<Option<String>> = Mutex::new(None);
  pub static ref ADMIN_REFRESH_TOKEN: Mutex<Option<String>> = Mutex::new(None);
}

pub struct TestClient {
  app: Router,
}

impl TestClient {
  pub fn new(app: Router) -> Self {
    Self { app }
  }

  pub async fn get_response(&self, request: Request<Body>) -> Response {
    self.app.clone().oneshot(request).await.unwrap()
  }

  pub async fn get_body(&self, response: Response) -> Value {
    let body: Bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
  }

  pub async fn call_request<F>(
    &self,
    endpoint_fn: F,
    token: String,
    expected_status: StatusCode,
    expected_body: Value,
  ) where
    F: Fn(String) -> Request<Body>,
  {
    let request: Request<Body> = endpoint_fn(token);
    let response: Response<Body> = self.get_response(request).await;
    assert_eq!(response.status(), expected_status);
    let body: Value = self.get_body(response).await;
    assert_eq!(body, expected_body);
  }

  pub async fn call_unauthorized_request<F>(
    &self,
    endpoint_fn: F,
    token: String,
    expected_body: Value,
  ) where
    F: Fn(String) -> Request<Body>,
  {
    self
      .call_request(endpoint_fn, token, StatusCode::UNAUTHORIZED, expected_body)
      .await;
  }

  pub async fn call_permission_denied_request<F>(
    &self,
    endpoint_fn: F,
    token: String,
    expected_body: Value,
  ) where
    F: Fn(String) -> Request<Body>,
  {
    self
      .call_request(endpoint_fn, token, StatusCode::FORBIDDEN, expected_body)
      .await;
  }
}

pub struct AuthSetup;

impl AuthSetup {
  pub async fn init() -> (Router, Arc<AuthState>, Arc<UserState>) {
    let config: AppConfig = APP_CONFIG.clone();
    let db_pool: PostgresDatabase = PostgresDatabase::connect(config.postgres).await.unwrap();
    let jwt_manager: JwtManager = JwtManager::new(config.jwt);
    let auth_state: Arc<AuthState> =
      Arc::new(AuthState::new(&db_pool, jwt_manager, &config.aws, &config.oauth2).await);
    let user_state: Arc<UserState> = Arc::new(UserState::new(&db_pool, &config.aws).await);
    let app: Router = AuthController::configure(auth_state.clone());
    (app, auth_state, user_state)
  }

  pub async fn initialize_user_tokens() {
    let mut access_token_guard: MutexGuard<Option<String>> = USER_ACCESS_TOKEN.lock().unwrap();
    let mut refresh_token_guard: MutexGuard<Option<String>> = USER_REFRESH_TOKEN.lock().unwrap();
    let mut user_id_guard: MutexGuard<Option<Uuid>> = USER_DEFAULT_USER_ID.lock().unwrap();

    if access_token_guard.is_none() && refresh_token_guard.is_none() {
      let (app, auth_state, _) = Self::init().await;
      let sign_up_payload = AuthSignUpRequest {
        email: USER_DEFAULT_EMAIL.clone(),
        password: USER_DEFAULT_PASSWORD.clone(),
        name: Option::from(rand::random::<FullName>().to_string()),
      };

      let user: UserEntity = auth_state.manager.sign_up(sign_up_payload).await.unwrap();

      let payload = AuthSignInRequest {
        email: USER_DEFAULT_EMAIL.clone(),
        password: USER_DEFAULT_PASSWORD.clone(),
      };

      let request: http::Request<Body> = AuthClient::sign_in(&payload);

      let test_client: TestClient = TestClient::new(app);
      let response: Response = test_client.get_response(request).await;
      let body: Value = test_client.get_body(response).await;

      let data: &Map<String, Value> = body.get("data").unwrap().as_object().unwrap();
      let access_token: Option<&Value> = data.get("access_token");
      let refresh_token: Option<&Value> = data.get("refresh_token");

      *access_token_guard = Some(access_token.unwrap().to_string());
      *refresh_token_guard = Some(refresh_token.unwrap().to_string());
      *user_id_guard = Some(user.id);
    }
  }

  pub async fn initialize_admin_tokens() {
    let mut access_token_guard: MutexGuard<Option<String>> = ADMIN_ACCESS_TOKEN.lock().unwrap();
    let mut refresh_token_guard: MutexGuard<Option<String>> = ADMIN_REFRESH_TOKEN.lock().unwrap();
    let mut user_id_guard: MutexGuard<Option<Uuid>> = USER_DEFAULT_USER_ID.lock().unwrap();

    if access_token_guard.is_none() && refresh_token_guard.is_none() {
      let (app, auth_state, user_state) = Self::init().await;
      let sign_up_payload = AuthSignUpRequest {
        email: ADMIN_DEFAULT_EMAIL.clone(),
        password: ADMIN_DEFAULT_PASSWORD.clone(),
        name: Option::from(rand::random::<FullName>().to_string()),
      };

      let user: UserEntity = auth_state.manager.sign_up(sign_up_payload).await.unwrap();

      user_state
        .manager
        .update_role(UserUpdateRoleRequest {
          user_id: user.id,
          role: UserRole::Admin,
        })
        .await
        .expect("TODO: panic message");

      let payload = AuthSignInRequest {
        email: ADMIN_DEFAULT_EMAIL.clone(),
        password: ADMIN_DEFAULT_PASSWORD.clone(),
      };

      let request: http::Request<Body> = AuthClient::sign_in(&payload);

      let test_client: TestClient = TestClient::new(app);
      let response: Response = test_client.get_response(request).await;
      let body: Value = test_client.get_body(response).await;

      let data: &Map<String, Value> = body.get("data").unwrap().as_object().unwrap();
      let access_token: Option<&Value> = data.get("access_token");
      let refresh_token: Option<&Value> = data.get("refresh_token");

      *access_token_guard = Some(access_token.unwrap().to_string());
      *refresh_token_guard = Some(refresh_token.unwrap().to_string());
      *user_id_guard = Some(user.id);
    }
  }

  pub fn fetch_token(token: &Mutex<Option<String>>) -> String {
    token
      .lock()
      .unwrap()
      .as_ref()
      .unwrap()
      .trim_matches('"')
      .to_string()
  }

  pub fn get_user_access_token() -> String {
    Self::fetch_token(&USER_ACCESS_TOKEN)
  }

  pub fn get_user_refresh_token() -> String {
    Self::fetch_token(&USER_REFRESH_TOKEN)
  }

  pub fn get_admin_access_token() -> String {
    Self::fetch_token(&ADMIN_ACCESS_TOKEN)
  }

  pub fn get_admin_refresh_token() -> String {
    Self::fetch_token(&ADMIN_REFRESH_TOKEN)
  }

  pub fn get_uuid_from_guard(guard: MutexGuard<Option<Uuid>>) -> Result<Uuid, &'static str> {
    match *guard {
      Some(ref uuid) => Ok(*uuid),
      None => Err("UUID is None"),
    }
  }
}
