use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct JobUserMessage {
  pub(crate) id: Uuid,
  pub(crate) verified: bool,
}

impl JobUserMessage {
  pub fn new(id: Uuid, verified: bool) -> Self {
    JobUserMessage { id, verified }
  }
}
