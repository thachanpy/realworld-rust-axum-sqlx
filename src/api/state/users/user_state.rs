use crate::api::manager::users::user_manager::UserManagerImpl;
use crate::api::repository::RepositoryImpl;
use crate::db::db::PostgresDatabase;
use crate::resources::config::AWSConfig;
use crate::service::aws::s3::s3::S3ServiceImpl;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserState {
  pub manager: UserManagerImpl,
}

impl UserState {
  pub async fn new(db_pool: &PostgresDatabase, aws_config: &AWSConfig) -> Self {
    let aws_config: AWSConfig = aws_config.clone();
    let manager: UserManagerImpl = UserManagerImpl::new(
      Arc::new(RepositoryImpl::new(db_pool.clone())),
      Arc::new(S3ServiceImpl::new(aws_config.region, aws_config.s3).await),
    );
    Self { manager }
  }
}
