use crate::api::client::users::request::user_request::{
  UserUpdateRoleRequest, UserUpdateStatusRequest,
};
use crate::api::client::users::response::user_response::UserResponse;
use crate::api::manager::users::user_response_converter::UserResponseConverter;
use crate::api::repository::users::entity::user_entity::UserEntity;
use crate::api::repository::users::repository::user_repository::UserRepository;
use crate::core::error::error::AppError;
use crate::core::request::pagination::Pagination;
use crate::core::request::sorting::Sorting;
use crate::service::aws::s3::s3::S3Service;
use crate::utils::datetime_utils::{TimeUtils, TimeUtilsBuilder};
use aws_smithy_types::error::metadata::ProvideErrorMetadata;
use axum::body::Bytes;
use axum::extract::Multipart;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

pub trait UserManager: Send + Sync + 'static {
  async fn all(
    &self,
    pagination: Pagination,
    sorting: Sorting,
  ) -> Result<Vec<UserResponse>, AppError>;
  async fn me(&self, id: Uuid) -> Result<UserEntity, AppError>;
  async fn upload_profile_url(
    &self,
    user_id: Uuid,
    file: Multipart,
  ) -> Result<Option<String>, AppError>;
  async fn update_role(&self, payload: UserUpdateRoleRequest) -> Result<(), AppError>;
  async fn update_status(&self, payload: UserUpdateStatusRequest) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct UserManagerImpl {
  user_repository: Arc<dyn UserRepository>,
  pub(crate) s3_service: Arc<dyn S3Service>,
}

impl UserManagerImpl {
  pub fn new(user_repository: Arc<dyn UserRepository>, s3_service: Arc<dyn S3Service>) -> Self {
    Self {
      user_repository,
      s3_service,
    }
  }
}

impl UserManager for UserManagerImpl {
  async fn all(
    &self,
    pagination: Pagination,
    sorting: Sorting,
  ) -> Result<Vec<UserResponse>, AppError> {
    let users: Vec<UserEntity> = self.user_repository.all(pagination, sorting).await?;
    Ok(self.users_response_converter(users).await)
  }

  async fn me(&self, user_id: Uuid) -> Result<UserEntity, AppError> {
    Ok(self.user_repository.me(user_id).await?)
  }

  async fn upload_profile_url(
    &self,
    user_id: Uuid,
    mut file: Multipart,
  ) -> Result<Option<String>, AppError> {
    let key: String = format!("users/{}/{}", user_id, "profile");
    let formatted_key: String = format!(
      "{}?t={}",
      key,
      TimeUtilsBuilder::new(TimeUtils::utc_now()).to_timestamp()
    );

    while let Some(field) = file.next_field().await.unwrap() {
      let data: Bytes = match field.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
          error!("Failed to read field bytes: {:?}", e);
          return Err(AppError::SomethingWentWrong);
        }
      };

      if let Err(e) = self.s3_service.upload_object(&key, data.to_vec()).await {
        error!("Failed to upload file: {:?}", e.message());
        return Err(AppError::SomethingWentWrong);
      }

      info!("File {:?} uploaded successfully", key);

      if let Err(e) = self
        .user_repository
        .update_profile_url(user_id, formatted_key.clone())
        .await
      {
        error!("Failed to update profile URL: {:?}", e);
        return Err(AppError::SomethingWentWrong);
      }
    }

    let url: Option<String> = self.s3_service.generate_presigned_url(&Some(key)).await?;
    Ok(url)
  }

  async fn update_role(&self, payload: UserUpdateRoleRequest) -> Result<(), AppError> {
    Ok(
      self
        .user_repository
        .update_role(payload.user_id, payload.role)
        .await?,
    )
  }

  async fn update_status(&self, payload: UserUpdateStatusRequest) -> Result<(), AppError> {
    self
      .user_repository
      .update_status(payload.user_id, payload.status)
      .await
  }
}
