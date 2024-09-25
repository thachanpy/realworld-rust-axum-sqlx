use aws_sdk_s3::operation::put_object::PutObjectError;
use aws_sdk_sqs::operation::send_message::SendMessageError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use tracing::error;

#[derive(Serialize)]
struct ErrorResponse {
  error: String,
  code: u16,
}

trait ApiError {
  fn status_code(&self) -> StatusCode;
  fn error_message(&self) -> String;

  fn to_response(&self) -> Response {
    let status: StatusCode = self.status_code();
    let error_response = ErrorResponse {
      error: self.error_message(),
      code: status.as_u16(),
    };

    (status, Json(error_response)).into_response()
  }
}

#[derive(Debug)]
pub enum AppError {
  InvalidJwtToken,
  PermissionDenied,
  InvalidOauth2Provider,
  UserNotFound,
  UserExistingEmail,
  UserPasswordIncorrect,
  SomethingWentWrong,
}

impl ApiError for AppError {
  fn status_code(&self) -> StatusCode {
    match self {
      AppError::InvalidJwtToken => StatusCode::UNAUTHORIZED,
      AppError::PermissionDenied => StatusCode::FORBIDDEN,
      AppError::InvalidOauth2Provider => StatusCode::BAD_REQUEST,
      AppError::UserNotFound => StatusCode::NOT_FOUND,
      AppError::UserExistingEmail => StatusCode::CONFLICT,
      AppError::UserPasswordIncorrect => StatusCode::UNAUTHORIZED,
      AppError::SomethingWentWrong => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }

  fn error_message(&self) -> String {
    match self {
      AppError::InvalidJwtToken => "invalid jwt token",
      AppError::PermissionDenied => "permission denied",
      AppError::InvalidOauth2Provider => "invalid oauth2 provider",
      AppError::UserNotFound => "user not found",
      AppError::UserExistingEmail => "existing email",
      AppError::UserPasswordIncorrect => "incorrect password",
      AppError::SomethingWentWrong => "something went wrong",
    }
    .to_string()
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    self.to_response()
  }
}

impl From<sqlx::Error> for AppError {
  fn from(err: sqlx::Error) -> Self {
    error!(%err, "database error occurred");
    AppError::SomethingWentWrong
  }
}

impl From<PutObjectError> for AppError {
  fn from(err: PutObjectError) -> Self {
    error!(%err, "failed to upload file to s3");
    AppError::SomethingWentWrong
  }
}

impl From<SendMessageError> for AppError {
  fn from(_: SendMessageError) -> Self {
    AppError::SomethingWentWrong
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::body::Body;
  use axum::response::{IntoResponse, Response};
  use http::StatusCode;

  #[tokio::test]
  async fn test_status_code() {
    assert_eq!(
      AppError::InvalidJwtToken.status_code(),
      StatusCode::UNAUTHORIZED
    );
    assert_eq!(AppError::UserNotFound.status_code(), StatusCode::NOT_FOUND);
    assert_eq!(
      AppError::UserExistingEmail.status_code(),
      StatusCode::CONFLICT
    );
    assert_eq!(
      AppError::UserPasswordIncorrect.status_code(),
      StatusCode::UNAUTHORIZED
    );
    assert_eq!(
      AppError::SomethingWentWrong.status_code(),
      StatusCode::INTERNAL_SERVER_ERROR
    );
  }

  #[tokio::test]
  async fn test_error_message() {
    assert_eq!(
      AppError::InvalidJwtToken.error_message(),
      "invalid jwt token"
    );
    assert_eq!(AppError::UserNotFound.error_message(), "user not found");
    assert_eq!(
      AppError::UserExistingEmail.error_message(),
      "existing email"
    );
    assert_eq!(
      AppError::UserPasswordIncorrect.error_message(),
      "incorrect password"
    );
    assert_eq!(
      AppError::SomethingWentWrong.error_message(),
      "something went wrong"
    );
  }

  #[tokio::test]
  async fn test_to_response() {
    let error = AppError::InvalidJwtToken;
    let response: Response = error.to_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }

  #[tokio::test]
  async fn test_into_response() {
    let error = AppError::InvalidJwtToken;
    let response: Response<Body> = error.into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }
}
