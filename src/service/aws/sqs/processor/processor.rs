use crate::core::error::error::AppError;
use crate::resources::config::AWSSQSConfig;
use async_trait::async_trait;
use aws_sdk_sqs::Client;
use serde_json::Value;
use std::sync::Arc;

#[async_trait]
pub trait SQSProcessor: Send + Sync + 'static {
  async fn process(&self, message: Value) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct SQSProcessorImpl<P> {
  client: Client,
  sqs_config: AWSSQSConfig,
  processor: P,
}

#[async_trait]
impl<P> SQSProcessor for Arc<P>
where
  P: SQSProcessor,
{
  async fn process(&self, message: Value) -> Result<(), AppError> {
    self.as_ref().process(message).await
  }
}
