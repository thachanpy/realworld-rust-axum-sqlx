use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum JobKind {
  Local,
}

impl JobKind {
  pub fn as_str(&self) -> &str {
    match self {
      JobKind::Local => "local",
    }
  }
}
