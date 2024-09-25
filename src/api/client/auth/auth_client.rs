use crate::api::client::auth::request::auth_request::{AuthSignInRequest, AuthSignUpRequest};
use crate::api::repository::users::constant::user_constant::OAuth2Provider;
use axum::body::Body;
use axum::extract::Request;
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json;

pub(crate) struct AuthClient;

impl AuthClient {
  pub fn sign_up(payload: &AuthSignUpRequest) -> Request<Body> {
    Request::builder()
      .method("POST")
      .uri("/auth/sign-up")
      .header(CONTENT_TYPE, "application/json")
      .body(Body::from(serde_json::to_string(payload).unwrap()))
      .unwrap()
  }

  pub fn sign_in(payload: &AuthSignInRequest) -> Request<Body> {
    Request::builder()
      .method("POST")
      .uri("/auth/sign-in")
      .header(CONTENT_TYPE, "application/json")
      .body(Body::from(serde_json::to_string(payload).unwrap()))
      .unwrap()
  }

  pub fn sign_in_oauth2_get_redirect_uri(provider: &OAuth2Provider) -> Request<Body> {
    Request::builder()
      .method("GET")
      .uri(format!("/auth/sign-in/{}", provider.as_str()))
      .body(Body::empty())
      .unwrap()
  }

  pub fn sign_out(access_token: String) -> Request<Body> {
    Request::builder()
      .method("POST")
      .uri("/auth/sign-out")
      .header(CONTENT_TYPE, "application/json")
      .header(AUTHORIZATION, format!("Bearer {}", access_token))
      .body(Body::empty())
      .unwrap()
  }

  pub fn refresh_token(refresh_token: String) -> Request<Body> {
    Request::builder()
      .method("POST")
      .uri("/auth/refresh-token")
      .header(CONTENT_TYPE, "application/json")
      .header(AUTHORIZATION, format!("Bearer {}", refresh_token))
      .body(Body::empty())
      .unwrap()
  }
}
