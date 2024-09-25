use crate::resources::config::CorsConfig;
use http::HeaderValue;
use tower_http::cors::{Any, CorsLayer};

pub struct Cors;

impl Cors {
  pub fn create_cors_layer(config: CorsConfig) -> CorsLayer {
    CorsLayer::new()
      .allow_origin(
        HeaderValue::from_str(config.allowed_origin.as_str()).expect("Invalid header value"),
      )
      .allow_methods(Any)
      .allow_headers(Any)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::body::Body;
  use axum::extract::Request;
  use axum::response::Response;
  use axum::routing::get;
  use axum::Router;
  use http::StatusCode;
  use tower::ServiceExt;

  const ALLOWED_ORIGIN: &str = "https://example.com";
  const NOT_ALLOWED_ORIGIN: &str = "https://notallowed.com";
  const ALLOW_ORIGIN_HEADER: &str = "access-control-allow-origin";

  fn create_app(config: CorsConfig) -> Router {
    let cors_layer: CorsLayer = Cors::create_cors_layer(config);
    Router::new()
      .route("/", get(|| async { "Hello, World!" }))
      .layer(cors_layer)
  }
  #[tokio::test]
  async fn test_cors_layer() {
    let config = CorsConfig {
      allowed_origin: ALLOWED_ORIGIN.to_string(),
    };
    let app = create_app(config);

    let response: Response = app
      .oneshot(
        Request::builder()
          .uri("/")
          .header("Origin", ALLOWED_ORIGIN)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
      response.headers().get(ALLOW_ORIGIN_HEADER).unwrap(),
      ALLOWED_ORIGIN
    );
  }

  #[tokio::test]
  async fn test_cors_rejects_not_allowed_origin() {
    let config = CorsConfig {
      allowed_origin: ALLOWED_ORIGIN.to_string(),
    };
    let app = create_app(config);

    let response = app
      .oneshot(
        Request::builder()
          .uri("/")
          .header("Origin", NOT_ALLOWED_ORIGIN)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_ne!(
      response
        .headers()
        .get(ALLOW_ORIGIN_HEADER)
        .and_then(|hv| hv.to_str().ok()),
      Some(NOT_ALLOWED_ORIGIN)
    );
  }
}
