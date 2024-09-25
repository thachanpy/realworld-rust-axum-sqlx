use crate::core::error::error::AppError;
use crate::job::message::JobMessage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::error;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum JobEventType {
  UserEvent,
}

impl JobEventType {
  pub fn as_str(&self) -> &str {
    match self {
      JobEventType::UserEvent => "user_event",
    }
  }

  pub fn from_str(s: &str) -> Result<Self, AppError> {
    match s {
      "user_event" => Ok(JobEventType::UserEvent),
      _ => Err(AppError::SomethingWentWrong),
    }
  }
}

pub struct JobEvent {}

impl JobEvent {
  pub async fn get_job_event(message: &Value) -> Result<(JobEventType, Value), AppError> {
    let json_message: Value = if message.is_string() {
      serde_json::from_str(message.as_str().unwrap()).map_err(|e| {
        eprintln!("Failed to parse JSON string: {:?}", e);
        AppError::SomethingWentWrong
      })?
    } else {
      message.clone()
    };

    let job_message: JobMessage = serde_json::from_value(json_message).map_err(|e| {
      error!("Failed to parse message: {:?}", e);
      AppError::SomethingWentWrong
    })?;

    if job_message.event_type.is_empty() {
      return Err(AppError::SomethingWentWrong);
    }

    let job_event_type: JobEventType =
      JobEventType::from_str(&job_message.event_type).map_err(|e| {
        error!("Failed to parse JobEventType: {:?}", e);
        AppError::SomethingWentWrong
      })?;

    Ok((job_event_type, job_message.data))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[tokio::test]
  async fn test_get_job_event_with_valid_string_message() {
    let message: Value =
      json!(r#"{"event_type": "user_event", "data": {"id": 1, "verified": true}}"#);
    let result: Result<(JobEventType, Value), AppError> = JobEvent::get_job_event(&message).await;

    assert!(result.is_ok());
    let (job_event_type, data) = result.unwrap();
    assert_eq!(job_event_type, JobEventType::UserEvent);
    assert_eq!(data, json!({"id": 1, "verified": true}));
  }

  #[tokio::test]
  async fn test_get_job_event_with_valid_object_message() {
    let message: Value = json!({
        "event_type": "user_event",
        "data": {
            "id": 1,
            "verified": true
        }
    });
    let result: Result<(JobEventType, Value), AppError> = JobEvent::get_job_event(&message).await;

    assert!(result.is_ok());
    let (job_event_type, data) = result.unwrap();
    assert_eq!(job_event_type, JobEventType::UserEvent);
    assert_eq!(data, json!({"id": 1, "verified": true}));
  }

  #[tokio::test]
  async fn test_get_job_event_with_invalid_string_message() {
    let message: Value = json!(r#"invalid json string"#);
    let result: Result<(JobEventType, Value), AppError> = JobEvent::get_job_event(&message).await;

    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_get_job_event_with_empty_event_type() {
    let message: Value = json!({
        "event_type": "",
        "data": {
            "id": 1,
            "verified": true
        }
    });
    let result: Result<(JobEventType, Value), AppError> = JobEvent::get_job_event(&message).await;

    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_get_job_event_with_invalid_event_type() {
    let message: Value = json!({
        "event_type": "invalid_event",
        "data": {
            "id": 1,
            "verified": true
        }
    });
    let result: Result<(JobEventType, Value), AppError> = JobEvent::get_job_event(&message).await;

    assert!(result.is_err());
  }
}
