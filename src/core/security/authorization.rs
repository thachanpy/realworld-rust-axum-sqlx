use crate::api::repository::users::constant::user_constant::UserRole;
use crate::core::error::error::AppError;
use crate::core::security::authentication::AccessTokenAuth;
use axum::body::Body;
use axum::extract::Request;
use axum::{middleware::Next, response::IntoResponse};

pub struct Authorization;

impl Authorization {
  pub async fn admin(
    access_token_auth: Result<AccessTokenAuth, AppError>,
    req: Request<Body>,
    next: Next,
  ) -> impl IntoResponse {
    Self::handle_role_based_access(access_token_auth, req, next, &[UserRole::Admin]).await
  }

  pub async fn user(
    access_token_auth: Result<AccessTokenAuth, AppError>,
    req: Request<Body>,
    next: Next,
  ) -> impl IntoResponse {
    Self::handle_role_based_access(access_token_auth, req, next, &[UserRole::User]).await
  }

  async fn handle_role_based_access(
    access_token_auth: Result<AccessTokenAuth, AppError>,
    req: Request<Body>,
    next: Next,
    required_roles: &[UserRole],
  ) -> impl IntoResponse {
    match access_token_auth {
      Ok(auth) => {
        if required_roles.contains(&auth.0.role) {
          Ok(next.run(req).await)
        } else {
          Err(AppError::PermissionDenied)
        }
      }
      Err(_) => Err(AppError::InvalidJwtToken),
    }
  }
}
