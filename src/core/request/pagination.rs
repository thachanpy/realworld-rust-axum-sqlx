use axum::extract::rejection::QueryRejection;
use axum::{
  async_trait,
  extract::{FromRequestParts, Query},
  http::request::Parts,
  http::StatusCode,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Pagination {
  pub page: Option<usize>,
  pub per_page: Option<usize>,
}

const DEFAULT_PAGE: usize = 1;
const DEFAULT_PER_PAGE: usize = 10;

#[async_trait]
impl<S> FromRequestParts<S> for Pagination
where
  S: Send + Sync,
{
  type Rejection = (StatusCode, String);

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let query: Result<Query<Pagination>, QueryRejection> =
      Query::<Pagination>::from_request_parts(parts, _state).await;

    match query {
      Ok(Query(pagination)) => {
        let page: usize = pagination.page.unwrap_or(DEFAULT_PAGE).max(1);
        let per_page: usize = pagination.per_page.unwrap_or(DEFAULT_PER_PAGE).max(0);

        Ok(Pagination {
          page: Some(page),
          per_page: Some(per_page),
        })
      }
      Err(_) => Err((
        StatusCode::BAD_REQUEST,
        "Invalid pagination parameters".to_string(),
      )),
    }
  }
}
