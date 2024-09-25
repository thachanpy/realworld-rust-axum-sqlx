use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct JobMessage {
  pub(crate) event_type: String,
  pub(crate) data: Value,
}

impl JobMessage {
  pub fn new(event_type: &str, data: impl Serialize) -> Self {
    let data: Value = serde_json::to_value(data).unwrap();
    JobMessage {
      event_type: event_type.to_string(),
      data,
    }
  }
  pub fn to_value(&self) -> Value {
    serde_json::to_value(self).unwrap()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::job::event::users::user_event::JobUserMessage;
  use serde_json::json;
  use uuid::Uuid;

  #[test]
  fn test_job_message_with_string_data() {
    let event_type: &str = "user_event";
    let data: &str = "some data";

    let message: JobMessage = JobMessage::new(event_type, data);

    assert_eq!(message.event_type, event_type);
    assert_eq!(message.data, json!(data));
  }

  #[test]
  fn test_job_message_with_struct_data() {
    let event_type: &str = "user_event";
    let id: Uuid = Uuid::new_v4();
    let data = JobUserMessage { id, verified: true };

    let message: JobMessage = JobMessage::new(event_type, data);

    assert_eq!(message.event_type, event_type);
    assert_eq!(message.data, json!({"id": id, "verified": true}));
  }

  #[test]
  fn test_job_message_to_value() {
    let event_type: &str = "user_event";
    let data: &str = "some data";

    let message: JobMessage = JobMessage::new(event_type, data);
    let value: Value = message.to_value();

    assert_eq!(
      value,
      json!({
          "event_type": event_type,
          "data": "some data"
      })
    );
  }

  #[test]
  fn test_job_message_serialization() {
    let event_type: &str = "user_event";
    let data: &str = "some data";

    let message: JobMessage = JobMessage::new(event_type, data);
    let serialized: String = serde_json::to_string(&message).unwrap();
    let expected: &str = r#"{"event_type":"user_event","data":"some data"}"#;

    assert_eq!(serialized, expected);
  }

  #[test]
  fn test_job_message_deserialization() {
    let json_data: &str = r#"{"event_type":"user_event","data":"some data"}"#;
    let message: JobMessage = serde_json::from_str(json_data).unwrap();

    assert_eq!(message.event_type, "user_event");
    assert_eq!(message.data, json!("some data"));
  }
}
