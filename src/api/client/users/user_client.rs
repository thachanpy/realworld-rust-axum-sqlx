use crate::api::client::users::request::user_request::UserUpdateRoleRequest;
use axum::body::Body;
use axum::extract::Request;
use http::header::{AUTHORIZATION, CONTENT_TYPE};

pub struct UserClient;

impl UserClient {
  pub fn all(access_token: String) -> Request<Body> {
    Request::builder()
      .method("GET")
      .uri("/users")
      .header(AUTHORIZATION, format!("Bearer {}", access_token))
      .body(Body::empty())
      .unwrap()
  }

  pub fn me(access_token: String) -> Request<Body> {
    Request::builder()
      .method("GET")
      .uri("/users/me")
      .header(AUTHORIZATION, format!("Bearer {}", access_token))
      .body(Body::empty())
      .unwrap()
  }

  pub fn update_profile_url(body: String, boundary: String, access_token: String) -> Request<Body> {
    http::Request::builder()
      .uri("/users/profile")
      .method("POST")
      .header(AUTHORIZATION, format!("Bearer {}", access_token))
      .header(
        CONTENT_TYPE,
        format!("multipart/form-data; boundary={}", boundary),
      )
      .body(Body::from(body))
      .unwrap()
  }

  pub fn update_role(payload: &UserUpdateRoleRequest, access_token: String) -> Request<Body> {
    Request::builder()
      .method("PUT")
      .uri("/users/role")
      .header(AUTHORIZATION, format!("Bearer {}", access_token))
      .header(CONTENT_TYPE, "application/json")
      .body(Body::from(serde_json::to_string(payload).unwrap()))
      .unwrap()
  }
}
