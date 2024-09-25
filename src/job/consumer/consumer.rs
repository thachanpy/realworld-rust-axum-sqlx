use crate::core::error::error::AppError;
use crate::job::kind::JobKind;
use crate::resources::config::AWSSQSConfig;
use crate::service::aws::sqs::consumer::consumer::SQSConsumerImpl;
use crate::service::aws::sqs::processor::processor::SQSProcessor;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info};

#[async_trait]
pub trait JobConsumer: Send + Sync {
  async fn start(&self, kind: &'static JobKind) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct JobConsumerImpl<P> {
  region: String,
  sqs_config: AWSSQSConfig,
  processor: Arc<P>,
}

impl<P> JobConsumerImpl<P>
where
  P: SQSProcessor + 'static,
{
  pub fn new(region: String, sqs_config: AWSSQSConfig, processor: P) -> Self {
    JobConsumerImpl {
      region,
      sqs_config,
      processor: Arc::new(processor),
    }
  }
}

#[async_trait]
impl<P> JobConsumer for JobConsumerImpl<P>
where
  P: SQSProcessor + 'static,
{
  async fn start(&self, kind: &'static JobKind) -> Result<(), AppError> {
    let job_kind: &str = kind.as_str();

    let num_replicas: u16 = self
      .sqs_config
      .jobs
      .get(job_kind)
      .ok_or(AppError::SomethingWentWrong)?
      .replicas;

    for i in 0..num_replicas {
      let region: String = self.region.clone();
      let sqs_config: AWSSQSConfig = self.sqs_config.clone();
      let processor: Arc<P> = Arc::clone(&self.processor);

      tokio::spawn(async move {
        let consumer: SQSConsumerImpl<Arc<P>> =
          SQSConsumerImpl::new(region.clone(), sqs_config.clone(), processor).await;
        info!("SQS - {} - replicas {} is starting", job_kind, i + 1);
        if let Err(e) = consumer.start(job_kind).await {
          error!(
            "SQS - {} - failed to start consumer replica {}: {:?}",
            job_kind,
            i + 1,
            e
          );
        }
      });
    }
    Ok(())
  }
}
