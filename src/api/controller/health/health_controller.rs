use crate::core::error::error::AppError;
use crate::core::response::response::SuccessResponse;
use axum::routing::get;
use axum::{response::IntoResponse, Router};

pub struct HealthController;

impl HealthController {
  pub(crate) fn configure() -> Router {
    Router::new().route("/health", get(Self::health))
  }
}

impl HealthController {
  async fn health() -> Result<impl IntoResponse, AppError> {
    Ok(SuccessResponse {
      data: "success".into(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::api::test::test_context::TestClient;
  use axum::body::Body;
  use axum::http::{Request, StatusCode};
  use axum::response::Response;
  use serde_json::{json, Value};

  #[tokio::test]
  async fn test_health_success() {
    let app: Router = HealthController::configure();

    let request: Request<Body> = Request::builder()
      .method("GET")
      .uri("/health")
      .body(Body::empty())
      .unwrap();

    let test_client: TestClient = TestClient::new(app);
    let response: Response = test_client.get_response(request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = test_client.get_body(response).await;

    assert_eq!(body, json!({ "data": "success" }));
  }
}
