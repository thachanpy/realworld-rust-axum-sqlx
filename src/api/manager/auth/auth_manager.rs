use crate::api::client::auth::request::auth_request::{
  AuthSignInRequest, AuthSignUpRequest, OAuth2SignInRequest,
};
use crate::api::client::auth::response::auth_response::SignInResponse;
use crate::api::client::oauth2::response::oauth2_response::OAuth2UserInfo;
use crate::api::manager::auth::auth_response_converter::AuthResponseConverter;
use crate::api::repository::refresh_tokens::entity::refresh_tokens_entity::RefreshTokenEntity;
use crate::api::repository::refresh_tokens::repository::refresh_token_repository::RefreshTokenRepository;
use crate::api::repository::users::constant::user_constant::{OAuth2Provider, UserRole};
use crate::api::repository::users::entity::user_entity::UserEntity;
use crate::api::repository::users::repository::user_repository::UserRepository;
use crate::core::error::error::AppError;
use crate::core::security::jwt_manager::{JwtManager, TokenType};
use crate::job::event::event::JobEventType;
use crate::job::event::users::user_event::JobUserMessage;
use crate::job::kind::JobKind;
use crate::job::message::JobMessage;
use crate::service::aws::sqs::producer::producer::SQSProducer;
use crate::service::oauth2::oauth2::OAuth2Service;
use crate::utils::hash_utils::HashUtils;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

pub trait AuthManager: Send + Sync + 'static {
  async fn sign_up(&self, payload: AuthSignUpRequest) -> Result<UserEntity, AppError>;
  async fn sign_in(&self, payload: AuthSignInRequest) -> Result<SignInResponse, AppError>;
  async fn send_verify_user_event(&self, user_id: Uuid) -> ();
  async fn generate_tokens(&self, user_id: Uuid, user_role: UserRole) -> (String, String);
  async fn sign_in_oauth2_get_redirect_uri(
    &self,
    provider: OAuth2Provider,
  ) -> Result<String, AppError>;

  async fn sign_in_oauth2(
    &self,
    provider: OAuth2Provider,
    payload: OAuth2SignInRequest,
  ) -> Result<SignInResponse, AppError>;
  async fn sign_out(&self, jti: Uuid) -> Result<(), AppError>;
  async fn refresh_token(
    &self,
    jti: Uuid,
    user_id: Uuid,
    user_role: UserRole,
  ) -> Result<SignInResponse, AppError>;
}

#[derive(Clone)]
pub struct AuthManagerImpl {
  user_repository: Arc<dyn UserRepository>,
  refresh_token_repository: Arc<dyn RefreshTokenRepository>,
  jwt_manager: JwtManager,
  sqs_producer: Arc<dyn SQSProducer>,
  google_oauth2: Arc<dyn OAuth2Service>,
}

impl AuthManagerImpl {
  pub fn new(
    user_repository: Arc<dyn UserRepository>,
    refresh_token_repository: Arc<dyn RefreshTokenRepository>,
    jwt_manager: JwtManager,
    sqs_producer: Arc<dyn SQSProducer>,
    google_oauth2: Arc<dyn OAuth2Service>,
  ) -> Self {
    Self {
      user_repository,
      refresh_token_repository,
      jwt_manager,
      sqs_producer,
      google_oauth2,
    }
  }
}

impl AuthManager for AuthManagerImpl {
  async fn sign_up(&self, payload: AuthSignUpRequest) -> Result<UserEntity, AppError> {
    let lowercase_email: String = payload.email.to_lowercase();
    match self
      .user_repository
      .by_email(lowercase_email.clone())
      .await?
    {
      Some(_) => Err(AppError::UserExistingEmail),
      None => {
        let new_user: UserEntity = self
          .user_repository
          .create(
            lowercase_email,
            Some(HashUtils::hash_password(&payload.password)),
            payload.name,
            None,
            None,
          )
          .await?;
        self.send_verify_user_event(new_user.id).await;
        Ok(new_user)
      }
    }
  }

  async fn sign_in(&self, payload: AuthSignInRequest) -> Result<SignInResponse, AppError> {
    match self.user_repository.by_email(payload.email).await? {
      Some(user)
        if HashUtils::verify_password(&payload.password, user.password.as_ref().unwrap()) =>
      {
        let (access_token, refresh_token) = self.generate_tokens(user.id, user.role).await;
        self.user_repository.update_logged_in_at(user.id).await?;
        Ok(Self::auth_token_response_converter(
          access_token,
          Some(refresh_token),
        ))
      }
      Some(_) => Err(AppError::UserPasswordIncorrect),
      None => Err(AppError::UserNotFound),
    }
  }

  async fn send_verify_user_event(&self, user_id: Uuid) -> () {
    let message: Value = JobMessage::new(
      JobEventType::UserEvent.as_str(),
      JobUserMessage::new(user_id, true),
    )
    .to_value();
    let _ = self.sqs_producer.send(JobKind::Local.as_str(), message).await;
  }

  async fn generate_tokens(&self, user_id: Uuid, user_role: UserRole) -> (String, String) {
    let refresh_token_entity: RefreshTokenEntity =
      self.refresh_token_repository.create(user_id).await.unwrap();
    let access_token: String = self.jwt_manager.generate_jwt(
      refresh_token_entity.id,
      user_id,
      TokenType::AccessToken,
      user_role,
    );
    let refresh_token = self.jwt_manager.generate_jwt(
      refresh_token_entity.id,
      user_id,
      TokenType::RefreshToken,
      user_role,
    );
    (access_token, refresh_token)
  }

  async fn sign_in_oauth2_get_redirect_uri(
    &self,
    provider: OAuth2Provider,
  ) -> Result<String, AppError> {
    let oauth2: Arc<dyn OAuth2Service> = match provider {
      OAuth2Provider::Google => self.google_oauth2.clone(),
    };
    let redirect_url: String = oauth2.get_redirect_url().await?;
    Ok(redirect_url)
  }

  async fn sign_in_oauth2(
    &self,
    provider: OAuth2Provider,
    payload: OAuth2SignInRequest,
  ) -> Result<SignInResponse, AppError> {
    let oauth2_service: Arc<dyn OAuth2Service> = self.google_oauth2.clone();
    let user_info: OAuth2UserInfo = oauth2_service.sign_in(payload).await?;
    let user: UserEntity = match self
      .user_repository
      .by_auth_provider(user_info.sub.clone(), provider)
      .await?
    {
      Some(existing_user) => existing_user,
      None => {
        let new_user: UserEntity = self
          .user_repository
          .create(
            user_info.email,
            None,
            user_info.name,
            Some(user_info.sub),
            Some(provider),
          )
          .await?;
        self.send_verify_user_event(new_user.id).await;
        new_user
      }
    };
    let (access_token, refresh_token) = self.generate_tokens(user.id, user.role).await;
    self.user_repository.update_logged_in_at(user.id).await?;
    Ok(Self::auth_token_response_converter(
      access_token,
      Some(refresh_token),
    ))
  }

  async fn sign_out(&self, jti: Uuid) -> Result<(), AppError> {
    self.refresh_token_repository.delete(jti).await?;
    Ok(())
  }

  async fn refresh_token(
    &self,
    jti: Uuid,
    user_id: Uuid,
    user_role: UserRole,
  ) -> Result<SignInResponse, AppError> {
    let refresh_token_entity: Option<RefreshTokenEntity> =
      self.refresh_token_repository.by_id(jti).await?;

    match refresh_token_entity {
      Some(_) => {
        let access_token: String =
          self
            .jwt_manager
            .generate_jwt(jti, user_id, TokenType::AccessToken, user_role);

        Ok(Self::auth_token_response_converter(access_token, None))
      }
      None => Err(AppError::InvalidJwtToken),
    }
  }
}
