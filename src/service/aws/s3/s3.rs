use crate::core::error::error::AppError;
use crate::resources::config::AWSS3Config;
use async_trait::async_trait;
use aws_config::{Region, SdkConfig};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::put_object::{PutObjectError, PutObjectOutput};
use aws_sdk_s3::presigning::{PresignedRequest, PresigningConfig};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
use std::time::Duration;

#[async_trait]
pub trait S3Service: Send + Sync + 'static {
  async fn upload_object(
    &self,
    key: &String,
    bytes: Vec<u8>,
  ) -> Result<PutObjectOutput, SdkError<PutObjectError, HttpResponse>>;
  async fn generate_presigned_url(&self, key: &Option<String>) -> Result<Option<String>, AppError>;
}

#[derive(Clone)]
pub struct S3ServiceImpl {
  client: Client,
  s3_config: AWSS3Config,
}

impl S3ServiceImpl {
  pub async fn new(region: String, s3_config: AWSS3Config) -> Self {
    let config: SdkConfig = aws_config::from_env()
      .region(Region::new(region))
      .load()
      .await;
    let client: Client = Client::new(&config);

    S3ServiceImpl { client, s3_config }
  }
}

#[async_trait]
impl S3Service for S3ServiceImpl {
  async fn upload_object(
    &self,
    key: &String,
    bytes: Vec<u8>,
  ) -> Result<PutObjectOutput, SdkError<PutObjectError, HttpResponse>> {
    self
      .client
      .put_object()
      .bucket(&self.s3_config.bucket_name)
      .key(format!("{}/{}", &self.s3_config.bucket_name, &key))
      .body(ByteStream::from(bytes))
      .send()
      .await
  }

  async fn generate_presigned_url(&self, key: &Option<String>) -> Result<Option<String>, AppError> {
    if let Some(key) = key {
      let expires_in: Duration =
        Duration::from_secs(self.s3_config.presigned_url_expiration_seconds);
      let presigning_config: PresigningConfig = PresigningConfig::expires_in(expires_in).unwrap();
      let presigned_request: Result<PresignedRequest, AppError> = self
        .client
        .get_object()
        .bucket(&self.s3_config.bucket_name)
        .key(format!("{}/{}", &self.s3_config.bucket_name, &key))
        .presigned(presigning_config)
        .await
        .map_err(|_| AppError::SomethingWentWrong);

      Ok(Option::from(presigned_request?.uri().to_string()))
    } else {
      Ok(None)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::launcher::APP_CONFIG;
  use crate::resources::config::AppConfig;

  async fn setup() -> S3ServiceImpl {
    let config: AppConfig = APP_CONFIG.clone();
    S3ServiceImpl::new(config.aws.region, config.aws.s3).await
  }

  #[tokio::test]
  async fn test_s3_upload_object_success() {
    let s3_service: S3ServiceImpl = setup().await;

    let result: Result<PutObjectOutput, SdkError<PutObjectError>> = s3_service
      .upload_object(&"test/key".to_string(), vec![1, 2, 3, 4])
      .await;

    assert!(result.is_ok());
    let output: PutObjectOutput = result.unwrap();
    assert!(output.e_tag().is_some());
  }

  #[tokio::test]
  async fn test_generate_presigned_url_success() {
    let s3_service: S3ServiceImpl = setup().await;

    let result: Result<Option<String>, AppError> = s3_service
      .generate_presigned_url(&Some("test/key".to_string()))
      .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
  }
}
