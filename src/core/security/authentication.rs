use crate::core::error::error::AppError;
use crate::core::security::jwt_manager::{Claims, JwtManager, TokenType};
use crate::launcher::APP_CONFIG;
use crate::resources::config::AppConfig;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::RequestPartsExt;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use http::request::Parts;

pub struct AccessTokenAuth(pub Claims);
pub struct RefreshTokenAuth(pub Claims);

pub struct TokenValidator;

impl TokenValidator {
  async fn extract_and_validate<S>(
    parts: &mut Parts,
    token_type: TokenType,
  ) -> Result<Claims, AppError>
  where
    S: Send + Sync,
  {
    let TypedHeader(Authorization(bearer)) = parts
      .extract::<TypedHeader<Authorization<Bearer>>>()
      .await
      .map_err(|_| AppError::InvalidJwtToken)?;

    let config: AppConfig = APP_CONFIG.clone();
    let jwt_manager: JwtManager = JwtManager::new(config.jwt);

    jwt_manager
      .validate_jwt(bearer.token(), token_type)
      .ok_or(AppError::InvalidJwtToken)
  }
}

#[async_trait]
impl<S> FromRequestParts<S> for AccessTokenAuth
where
  S: Send + Sync,
{
  type Rejection = AppError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let claims: Claims =
      TokenValidator::extract_and_validate::<S>(parts, TokenType::AccessToken).await?;
    Ok(AccessTokenAuth(claims))
  }
}

#[async_trait]
impl<S> FromRequestParts<S> for RefreshTokenAuth
where
  S: Send + Sync,
{
  type Rejection = AppError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let claims: Claims =
      TokenValidator::extract_and_validate::<S>(parts, TokenType::RefreshToken).await?;
    Ok(RefreshTokenAuth(claims))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::api::repository::users::constant::user_constant::UserRole;
  use uuid::Uuid;

  #[tokio::test]
  async fn test_access_jwt_generation_and_validation() {
    let config: AppConfig = APP_CONFIG.clone();
    let jwt_manager: JwtManager = JwtManager::new(config.jwt);

    let jti: Uuid = Uuid::new_v4();
    let user_id: Uuid = Uuid::new_v4();

    let token_type = TokenType::AccessToken;
    let token: String = jwt_manager.generate_jwt(jti, user_id, token_type, UserRole::User);

    let claims: Option<Claims> = jwt_manager.validate_jwt(&token, token_type);
    assert!(matches!(claims, Some(claims) if claims.sub == user_id));
  }

  #[tokio::test]
  async fn test_refresh_jwt_generation_and_validation() {
    let config: AppConfig = APP_CONFIG.clone();
    let jwt_manager: JwtManager = JwtManager::new(config.jwt);

    let jti: Uuid = Uuid::new_v4();
    let user_id: Uuid = Uuid::new_v4();

    let token_type = TokenType::RefreshToken;
    let token: String = jwt_manager.generate_jwt(jti, user_id, token_type, UserRole::User);

    let claims: Option<Claims> = jwt_manager.validate_jwt(&token, token_type);
    assert!(matches!(claims, Some(claims) if claims.sub == user_id));
  }

  #[tokio::test]
  async fn test_invalid_access_jwt() {
    let config: AppConfig = APP_CONFIG.clone();
    let jwt_manager: JwtManager = JwtManager::new(config.jwt);

    let invalid_token: &str = "invalid_token";

    let claims: Option<Claims> = jwt_manager.validate_jwt(invalid_token, TokenType::AccessToken);
    assert!(matches!(claims, None));
  }

  #[tokio::test]
  async fn test_invalid_refresh_jwt() {
    let config: AppConfig = APP_CONFIG.clone();
    let jwt_manager: JwtManager = JwtManager::new(config.jwt);

    let invalid_token: &str = "invalid_token";

    let claims: Option<Claims> = jwt_manager.validate_jwt(invalid_token, TokenType::RefreshToken);
    assert!(matches!(claims, None));
  }
}
