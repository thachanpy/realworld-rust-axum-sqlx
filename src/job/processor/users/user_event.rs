use crate::api::client::users::request::user_request::UserUpdateStatusRequest;
use crate::api::manager::users::user_manager::UserManager;
use crate::api::repository::users::constant::user_constant::UserStatus;
use crate::api::state::users::user_state::UserState;
use crate::core::error::error::AppError;
use crate::job::event::event::{JobEvent, JobEventType};
use crate::job::event::users::user_event::JobUserMessage;
use crate::service::aws::sqs::processor::processor::SQSProcessor;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct UserEventProcessor {
  user_state: Arc<UserState>,
}

impl UserEventProcessor {
  pub async fn new(user_state: Arc<UserState>) -> Self {
    Self { user_state }
  }
}

#[async_trait]
impl SQSProcessor for UserEventProcessor {
  async fn process(&self, message: Value) -> Result<(), AppError> {
    let job_event: (JobEventType, Value) = JobEvent::get_job_event(&message).await?;
    if job_event.0.as_str() == JobEventType::UserEvent.as_str() {
      info!("Processing message: {:?}", message);
      let data: JobUserMessage =
        serde_json::from_str(&*job_event.1.to_string()).expect("Could not parse job user message");
      if data.verified == true {
        self
          .user_state
          .manager
          .update_status(UserUpdateStatusRequest {
            user_id: data.id,
            status: UserStatus::Verified,
          })
          .await
          .expect("Could not update user status");
      }
    }
    Ok(())
  }
}
