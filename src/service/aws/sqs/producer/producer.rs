use crate::core::error::error::AppError;
use crate::resources::config::{AWSSQSConfig, AWSSQSJobConfig};
use async_trait::async_trait;
use aws_config::{Region, SdkConfig};
use aws_sdk_sqs::operation::send_message::builders::SendMessageFluentBuilder;
use aws_sdk_sqs::Client;
use serde_json::Value;
use tracing::{error, info};

#[async_trait]
pub trait SQSProducer: Send + Sync + 'static {
  async fn send(&self, kind: &str, payload: Value) -> Result<(), AppError>;
}
#[derive(Clone)]
pub struct SQSProducerImpl {
  client: Client,
  sqs_config: AWSSQSConfig,
}

impl SQSProducerImpl {
  pub async fn new(region: String, sqs_config: AWSSQSConfig) -> Self {
    let config: SdkConfig = aws_config::from_env()
      .region(Region::new(region))
      .load()
      .await;
    let client: Client = Client::new(&config);
    SQSProducerImpl { client, sqs_config }
  }
}

#[async_trait]
impl SQSProducer for SQSProducerImpl {
  async fn send(&self, kind: &str, payload: Value) -> Result<(), AppError> {
    let client: Client = self.client.clone();
    let sqs_config: AWSSQSConfig = self.sqs_config.clone();
    let kind: String = kind.to_string();
    let payload: Value = payload.clone();

    tokio::spawn(async move {
      let config: AWSSQSJobConfig = match sqs_config.jobs.get(&kind) {
        Some(config) => config.clone(),
        None => {
          error!("SQS - {} - Producer not initialized", kind);
          return Err(AppError::SomethingWentWrong);
        }
      };

      let message_body: String = payload.to_string();

      let mut request: SendMessageFluentBuilder = client
          .send_message()
          .queue_url(&config.queue_url)
          .message_body(&message_body);

      if (1..=900).contains(&config.delay_seconds) {
        request = request.delay_seconds(config.delay_seconds);
      }

      match request.send().await {
        Ok(response) => {
          info!(
            "SQS - {} - send message {} successfully!",
            kind, message_body
          );
          Ok(response)
        }
        Err(error) => {
          error!(
            "SQS - {} - failed to send message {}: error {}",
            kind, message_body, error
          );
          Err(AppError::SomethingWentWrong)
        }
      }
    }).await??;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::launcher::APP_CONFIG;
  use crate::resources::config::AppConfig;
  use serde_json::json;

  async fn setup() -> SQSProducerImpl {
    let config: AppConfig = APP_CONFIG.clone();
    SQSProducerImpl::new(config.aws.region, config.aws.sqs).await
  }

  #[tokio::test]
  async fn test_sqs_producer_send_message_success() {
    let producer: SQSProducerImpl = setup().await;

    let payload: Value = json!({ "key": "value" });
    let result: Result<(), AppError> = producer.send("local", payload).await;

    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_sqs_producer_send_message_failure() {
    let producer: SQSProducerImpl = setup().await;

    let payload: Value = json!({ "key": "value" });
    let result: Result<(), AppError> = producer.send("fake_job", payload).await;

    assert!(result.is_err());
  }
}
