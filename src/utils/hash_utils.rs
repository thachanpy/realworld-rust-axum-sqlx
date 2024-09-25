use bcrypt::{hash, verify};

pub const COST: u32 = 8;

pub struct HashUtils;

impl HashUtils {
  pub fn hash_password(password: &str) -> String {
    hash(password, COST).unwrap()
  }

  pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    verify(password, hashed_password).unwrap_or(false)
  }
}

#[cfg(test)]
mod tests {
  use crate::utils::hash_utils::HashUtils;
  use bcrypt::verify;

  #[tokio::test]
  async fn test_hash_password() {
    let password: &str = "my_password";

    let hashed_password: String = HashUtils::hash_password(password);

    assert_ne!(password, hashed_password);
    assert!(verify(password, &hashed_password).expect("Failed to verify hash"));
  }

  #[tokio::test]
  async fn test_verify_password() {
    let password = "my_password";
    let hashed_password = HashUtils::hash_password(password);

    assert!(HashUtils::verify_password(password, &hashed_password));
    assert!(!HashUtils::verify_password(
      "wrong_password",
      &hashed_password
    ));
  }
}
