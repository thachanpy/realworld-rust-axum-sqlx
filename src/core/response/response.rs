use axum::http::StatusCode;
use axum::{response::IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct SuccessResponse<T: Serialize> {
  pub(crate) data: Option<T>,
}

impl<T: Serialize> IntoResponse for SuccessResponse<T> {
  fn into_response(self) -> axum::response::Response {
    (StatusCode::OK, Json(self)).into_response()
  }
}

#[derive(Serialize)]
pub(crate) struct CreatedResponse<T: Serialize> {
  pub(crate) message: Option<T>,
}

impl<T: Serialize> IntoResponse for CreatedResponse<T> {
  fn into_response(self) -> axum::response::Response {
    (StatusCode::CREATED, Json(self)).into_response()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::response::Response;

  #[tokio::test]
  async fn test_success_response() {
    let response_data = SuccessResponse {
      data: Some("test data".to_string()),
    };

    let response: Response = response_data.into_response();

    assert_eq!(response.status(), StatusCode::OK);
  }

  #[tokio::test]
  async fn test_created_response() {
    let response_message = CreatedResponse {
      message: Some("created".to_string()),
    };

    let response: Response = response_message.into_response();

    assert_eq!(response.status(), StatusCode::CREATED);
  }
}
