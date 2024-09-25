use serde::Serialize;

#[derive(Serialize)]
pub enum BaseMessage {
  #[serde(rename = "created")]
  Created,

  #[serde(rename = "updated")]
  Updated,

  #[serde(rename = "success")]
  Success,
}
