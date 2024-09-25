use axum::extract::rejection::QueryRejection;
use axum::{
  async_trait,
  extract::{FromRequestParts, Query},
  http::request::Parts,
  http::StatusCode,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Sorting {
  pub order_by: Option<String>,
  pub order_direction: Option<String>,
}

const DEFAULT_ORDER_BY: &str = "created_at";
const DEFAULT_ORDER_DIRECTION: &str = "asc";

#[async_trait]
impl<S> FromRequestParts<S> for Sorting
where
  S: Send + Sync,
{
  type Rejection = (StatusCode, String);

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let query: Result<Query<Sorting>, QueryRejection> =
      Query::<Sorting>::from_request_parts(parts, _state).await;

    match query {
      Ok(query) => {
        let order_by: String = query
          .order_by
          .clone()
          .unwrap_or_else(|| String::from(DEFAULT_ORDER_BY))
          .to_lowercase();
        let order_direction: String = query
          .order_direction
          .clone()
          .unwrap_or_else(|| String::from(DEFAULT_ORDER_DIRECTION))
          .to_lowercase();

        let order_direction: String = match order_direction.as_str() {
          "asc" | "desc" => order_direction,
          _ => String::from(DEFAULT_ORDER_DIRECTION),
        };

        Ok(Sorting {
          order_by: Some(order_by),
          order_direction: Some(order_direction),
        })
      }
      Err(_) => Err((
        StatusCode::BAD_REQUEST,
        "Invalid pagination parameters".to_string(),
      )),
    }
  }
}
