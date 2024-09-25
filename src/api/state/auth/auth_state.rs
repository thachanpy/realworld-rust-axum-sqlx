use crate::api::manager::auth::auth_manager::AuthManagerImpl;
use crate::api::repository::users::constant::user_constant::OAuth2Provider;
use crate::api::repository::RepositoryImpl;
use crate::core::security::jwt_manager::JwtManager;
use crate::db::db::PostgresDatabase;
use crate::resources::config::{AWSConfig, OAuth2Config};
use crate::service::aws::sqs::producer::producer::SQSProducerImpl;
use crate::service::oauth2::oauth2::OAuth2ServiceImpl;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthState {
  pub manager: AuthManagerImpl,
}

impl AuthState {
  pub async fn new(
    db_pool: &PostgresDatabase,
    jwt_manager: JwtManager,
    aws_config: &AWSConfig,
    oauth2_config: &HashMap<String, OAuth2Config>,
  ) -> Self {
    let aws_config: AWSConfig = aws_config.clone();

    let google_oauth2_config: OAuth2Config = oauth2_config
      .get(OAuth2Provider::Google.as_str())
      .unwrap()
      .clone();

    let manager: AuthManagerImpl = AuthManagerImpl::new(
      Arc::new(RepositoryImpl::new(db_pool.clone())),
      Arc::new(RepositoryImpl::new(db_pool.clone())),
      jwt_manager,
      Arc::new(SQSProducerImpl::new(aws_config.region, aws_config.sqs).await),
      Arc::new(OAuth2ServiceImpl::new(google_oauth2_config).await),
    );
    Self { manager }
  }
}
