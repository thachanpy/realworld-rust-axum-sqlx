use axum::body::Body;
use axum::extract::MatchedPath;
use axum::http::Request;
use tower_http::trace::{HttpMakeClassifier, TraceLayer};
use tracing::{info_span, Span};
use tracing_subscriber;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub struct Logging;

impl Logging {
  pub fn init_tracing() {
    tracing_subscriber::registry()
      .with(
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
          format!(
            "{}=debug,tower_http=debug,axum::rejection=trace",
            env!("CARGO_CRATE_NAME")
          )
          .into()
        }),
      )
      .with(tracing_subscriber::fmt::layer())
      .init();
  }

  pub fn create_trace_layer() -> TraceLayer<HttpMakeClassifier, fn(&Request<Body>) -> Span> {
    TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
      let matched_path: Option<&str> = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str);
      info_span!(
          "http_request",
          method = ?request.method(),
          matched_path,
          some_other_field = tracing::field::Empty,
      )
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::response::Response;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
  };
  use tower::ServiceExt;
  use tracing_subscriber::FmtSubscriber;

  #[tokio::test]
  async fn test_create_trace_layer() {
    let subscriber = FmtSubscriber::builder()
      .with_max_level(tracing::Level::DEBUG)
      .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    let app: Router = Router::new()
      .route("/test", get(|| async { "OK".into_response() }))
      .layer(Logging::create_trace_layer());

    let request: Request<Body> = Request::builder().uri("/test").body(Body::empty()).unwrap();

    let response: Response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }
}
