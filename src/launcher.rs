use crate::api::server::Api;
use crate::core::error::error::AppError;
use crate::resources::config::AppConfig;
use crate::worker::server::Worker;
use std::env;
use std::sync::LazyLock;

pub(crate) static APP_CONFIG: LazyLock<AppConfig> = LazyLock::new(|| AppConfig::load().unwrap());

pub enum ServiceType {
  API,
  WORKER,
}

impl ServiceType {
  pub(crate) fn from_str_case_insensitive(input: &str) -> Self {
    match input.to_lowercase().as_str() {
      "api" => ServiceType::API,
      "worker" => ServiceType::WORKER,
      _ => ServiceType::API,
    }
  }
}

pub struct ServiceLauncher {
  service_type: ServiceType,
  config: AppConfig,
}

impl ServiceLauncher {
  pub(crate) fn new() -> Self {
    let args: Vec<String> = env::args().collect();
    let service_type: ServiceType = if args.len() > 1 {
      ServiceType::from_str_case_insensitive(&args[1])
    } else {
      ServiceType::API
    };

    ServiceLauncher {
      service_type,
      config: APP_CONFIG.clone(),
    }
  }

  pub async fn start(&self) -> Result<(), AppError> {
    match self.service_type {
      ServiceType::API => self.start_api_service().await,
      ServiceType::WORKER => self.start_worker_service().await,
    }
  }

  async fn start_api_service(&self) -> Result<(), AppError> {
    let app: Api = Api::new(self.config.clone()).await?;
    app.start().await?;
    Ok(())
  }

  async fn start_worker_service(&self) -> Result<(), AppError> {
    let app: Worker = Worker::new(self.config.clone()).await?;
    app.start().await?;
    Ok(())
  }
}
