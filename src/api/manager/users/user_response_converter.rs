use crate::api::client::users::response::user_response::UserResponse;
use crate::api::manager::users::user_manager::UserManagerImpl;
use crate::api::repository::users::entity::user_entity::UserEntity;
use futures::future::join_all;

pub trait UserResponseConverter {
  async fn user_response_converter(&self, user: UserEntity) -> UserResponse;
  async fn users_response_converter(&self, users: Vec<UserEntity>) -> Vec<UserResponse>;
}

impl UserResponseConverter for UserManagerImpl {
  async fn user_response_converter(&self, user: UserEntity) -> UserResponse {
    let profile: Option<String> = match user.profile_url {
      Some(ref url) if url.starts_with("http://") || url.starts_with("https://") => {
        Some(url.clone())
      }
      Some(_) => self
        .s3_service
        .generate_presigned_url(&user.profile_url)
        .await
        .unwrap(),
      None => None,
    };

    UserResponse {
      id: user.id,
      email: user.email,
      name: user.name,
      role: user.role,
      profile_url: profile,
      logged_in_at: user.logged_in_at,
      created_at: user.created_at,
      updated_at: user.updated_at,
    }
  }

  async fn users_response_converter(&self, users: Vec<UserEntity>) -> Vec<UserResponse> {
    let futures: Vec<_> = users
      .into_iter()
      .map(|user| self.user_response_converter(user))
      .collect();

    let results: Vec<UserResponse> = join_all(futures).await;
    results
  }
}
