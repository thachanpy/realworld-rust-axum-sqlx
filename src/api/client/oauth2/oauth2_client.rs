use axum::body::Body;
use axum::extract::Request;
use http::header::AUTHORIZATION;

pub(crate) struct AuthClient;

impl AuthClient {
  pub fn get_user_info(url: String, access_token: String) -> http::Request<Body> {
    Request::get(url)
      .header(AUTHORIZATION, format!("Bearer {}", access_token))
      .body(Body::empty())
      .unwrap()
  }
}
