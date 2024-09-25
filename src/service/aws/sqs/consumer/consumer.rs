use crate::core::error::error::AppError;
use crate::resources::config::{AWSSQSConfig, AWSSQSJobConfig};
use crate::service::aws::sqs::processor::processor::SQSProcessor;
use async_trait::async_trait;
use aws_config::SdkConfig;
use aws_sdk_sqs::config::Region;
use aws_sdk_sqs::operation::delete_message::DeleteMessageOutput;
use aws_sdk_sqs::operation::receive_message::ReceiveMessageOutput;
use aws_sdk_sqs::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

#[async_trait]
pub trait SQSConsumer: Send + Sync + 'static {
  async fn start(&self, kind: &str) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct SQSConsumerImpl<P> {
  client: Client,
  sqs_config: AWSSQSConfig,
  processor: P,
}

impl<P> SQSConsumerImpl<P>
where
  P: SQSProcessor + 'static,
{
  pub async fn new(region: String, sqs_config: AWSSQSConfig, processor: P) -> Self {
    let config: SdkConfig = aws_config::from_env()
      .region(Region::new(region))
      .load()
      .await;
    let client: Client = Client::new(&config);
    SQSConsumerImpl {
      client,
      sqs_config,
      processor,
    }
  }
}

impl<P> SQSConsumerImpl<P>
where
  P: SQSProcessor + 'static,
{
  pub(crate) async fn start(&self, kind: &str) -> Result<(), AppError> {
    let config: AWSSQSJobConfig = match self.sqs_config.jobs.get(kind) {
      Some(config) => config.clone(),
      None => {
        error!("SQS - {} - Consumer not initialized", kind);
        return Err(AppError::SomethingWentWrong);
      }
    };
    loop {
      let messages: ReceiveMessageOutput = self
        .client
        .receive_message()
        .queue_url(&config.queue_url)
        .max_number_of_messages(config.max_number_of_messages)
        .wait_time_seconds(config.wait_time_seconds)
        .send()
        .await
        .unwrap();

      if let Some(messages) = messages.messages {
        for message in messages {
          if let Some(body) = &message.body {
            let json_body: Value = match serde_json::to_value(body) {
              Ok(value) if !value.is_null() => value,
              _ => {
                error!(
                  "SQS - {} - Failed to convert message body to valid JSON",
                  kind
                );
                continue;
              }
            };

            if let Err(e) = self.processor.process(json_body).await {
              error!("SQS - {} - Failed to process message: {:?}", kind, e);
            }
          }

          if let Some(receipt_handle) = message.receipt_handle {
            self
              .delete_message(kind, &config.queue_url, &receipt_handle)
              .await?;
          }
        }
        sleep(Duration::from_secs(1)).await;
      }
    }
  }

  async fn delete_message(
    &self,
    kind: &str,
    queue_url: &str,
    receipt_handle: &str,
  ) -> Result<DeleteMessageOutput, AppError> {
    info!(
      "SQS - {} - Attempting to delete message receipt handle: {}",
      kind, receipt_handle
    );

    match self
      .client
      .delete_message()
      .queue_url(queue_url)
      .receipt_handle(receipt_handle)
      .send()
      .await
    {
      Ok(response) => {
        info!(
          "SQS - {} - Successfully deleted message {}",
          kind, receipt_handle
        );
        Ok(response)
      }
      Err(e) => {
        error!(
          "SQS - {} - Failed to delete message with receipt handle: {}. Error: {:?}",
          kind, receipt_handle, e
        );
        Err(AppError::SomethingWentWrong)
      }
    }
  }
}
