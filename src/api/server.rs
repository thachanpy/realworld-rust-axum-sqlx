use crate::api::controller::auth::auth_controller::AuthController;
use crate::api::controller::health::health_controller::HealthController;
use crate::api::controller::users::user_controller::UserController;
use crate::api::state::auth::auth_state::AuthState;
use crate::api::state::users::user_state::UserState;
use crate::core::cors::cors::Cors;
use crate::core::error::error::AppError;
use crate::core::logging::logging::Logging;
use crate::core::security::jwt_manager::JwtManager;
use crate::db::db::PostgresDatabase;
use crate::resources::config::AppConfig;
use axum::body::Body;
use axum::http::Request;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::decompression::RequestDecompressionLayer;
use tower_http::trace::{HttpMakeClassifier, TraceLayer};
use tracing::{info, Span};

type LogsLayerType = TraceLayer<HttpMakeClassifier, fn(&Request<Body>) -> Span>;

pub struct Api {
  config: AppConfig,
  jwt_manager: JwtManager,
  db_pool: PostgresDatabase,
  cors_layer: CorsLayer,
  logs_layer: LogsLayerType,
}

impl Api {
  pub async fn new(config: AppConfig) -> Result<Self, AppError> {
    Logging::init_tracing();
    let jwt_manager: JwtManager = JwtManager::new(config.jwt.clone());
    let cors_layer: CorsLayer = Cors::create_cors_layer(config.server.api.cors.clone());
    let db_pool: PostgresDatabase = PostgresDatabase::connect(config.postgres.clone()).await?;
    let logs_layer: LogsLayerType = Logging::create_trace_layer();

    Ok(Self {
      config,
      jwt_manager,
      db_pool,
      cors_layer,
      logs_layer,
    })
  }

  async fn configure_routes(&self) -> Result<Router, AppError> {
    let auth_state: Arc<AuthState> = Arc::new(
      AuthState::new(
        &self.db_pool,
        self.jwt_manager.clone(),
        &self.config.aws,
        &self.config.oauth2,
      )
      .await,
    );
    let user_state: Arc<UserState> =
      Arc::new(UserState::new(&self.db_pool, &self.config.aws).await);

    let app: Router = Router::new()
      .nest(
        &self.config.server.path_prefix,
        Router::new()
          .merge(self.configure_health())
          .merge(self.configure_auth(auth_state))
          .merge(self.configure_user(user_state)),
      )
      .layer(self.cors_layer.clone())
      .layer(self.logs_layer.clone())
      .layer(RequestDecompressionLayer::new())
      .layer(CompressionLayer::new());

    Ok(app)
  }

  fn configure_health(&self) -> Router {
    HealthController::configure()
  }

  fn configure_auth(&self, auth_state: Arc<AuthState>) -> Router {
    AuthController::configure(auth_state)
  }

  fn configure_user(&self, user_state: Arc<UserState>) -> Router {
    UserController::configure(user_state)
  }

  pub async fn start(self) -> Result<(), AppError> {
    let app: Router = self.configure_routes().await?;
    let addr: String = format!(
      "{}:{}",
      self.config.server.address, self.config.server.api.port
    );
    let listener: TcpListener = TcpListener::bind(addr).await.unwrap();
    info!("Server is started on {}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service())
      .await
      .map_err(|_| AppError::SomethingWentWrong)
  }
}
