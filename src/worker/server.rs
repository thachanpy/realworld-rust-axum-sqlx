use crate::api::controller::health::health_controller::HealthController;
use crate::api::state::users::user_state::UserState;
use crate::core::error::error::AppError;
use crate::core::logging::logging::Logging;
use crate::db::db::PostgresDatabase;
use crate::job::consumer::consumer::{JobConsumer, JobConsumerImpl};
use crate::job::kind::JobKind;
use crate::job::processor::users::user_event::UserEventProcessor;
use crate::resources::config::AppConfig;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

pub struct Worker {
  config: AppConfig,
  db_pool: PostgresDatabase,
}

impl Worker {
  pub async fn new(config: AppConfig) -> Result<Self, AppError> {
    Logging::init_tracing();
    let db_pool: PostgresDatabase = PostgresDatabase::connect(config.postgres.clone()).await?;
    Ok(Self { config, db_pool })
  }

  async fn init_user_event_processor(&self) -> Result<UserEventProcessor, AppError> {
    let user_state: UserState = UserState::new(&self.db_pool, &self.config.aws).await;
    let processor: UserEventProcessor = UserEventProcessor::new(Arc::new(user_state)).await;
    Ok(processor)
  }

  async fn setup_job_consumer(
    &self,
    processor: UserEventProcessor,
  ) -> JobConsumerImpl<UserEventProcessor> {
    JobConsumerImpl::new(
      self.config.aws.region.clone(),
      self.config.aws.sqs.clone(),
      processor,
    )
  }

  fn configure_routes(&self) -> Router {
    Router::new().nest(
      &self.config.server.path_prefix,
      Router::new().merge(self.configure_health()),
    )
  }

  fn configure_health(&self) -> Router {
    HealthController::configure()
  }

  pub async fn start(self) -> Result<(), AppError> {
    let processor: UserEventProcessor = self.init_user_event_processor().await?;
    let consumer: JobConsumerImpl<UserEventProcessor> = self.setup_job_consumer(processor).await;
    consumer.start(&JobKind::Local).await?;

    let app: Router = self.configure_routes();
    let addr: String = format!(
      "{}:{}",
      self.config.server.address, self.config.server.worker.port
    );
    let listener: TcpListener = TcpListener::bind(addr).await.unwrap();
    info!("Server is started on {}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service())
      .await
      .map_err(|_| AppError::SomethingWentWrong)
  }
}
