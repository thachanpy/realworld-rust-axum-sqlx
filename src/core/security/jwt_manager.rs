use crate::api::repository::users::constant::user_constant::UserRole;
use crate::resources::config::JwtConfig;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use jsonwebtoken::{
  decode, encode, get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
  Validation,
};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum TokenType {
  #[serde(rename = "access_token")]
  AccessToken,
  #[serde(rename = "refresh_token")]
  RefreshToken,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Claims {
  pub jti: Uuid,
  pub token_type: TokenType,
  pub sub: Uuid,
  pub exp: usize,
  pub role: UserRole,
}

#[derive(Clone)]
pub struct JwtManager {
  private_key_base64: String,
  public_key_base64: String,
  access_expiration_seconds: i64,
  refresh_expiration_seconds: i64,
  algorithm: Algorithm,
}

impl JwtManager {
  const DEFAULT_EXPIRATION: usize = 0;

  pub fn new(config: JwtConfig) -> Self {
    JwtManager {
      private_key_base64: config.private_key_base64,
      public_key_base64: config.public_key_base64,
      access_expiration_seconds: config.access_expiration_seconds,
      refresh_expiration_seconds: config.refresh_expiration_seconds,
      algorithm: config.algorithm,
    }
  }

  fn decode_key_base64(key_base64: String) -> Vec<u8> {
    BASE64_STANDARD.decode(key_base64).unwrap()
  }

  pub fn load_private_key(&self) -> EncodingKey {
    EncodingKey::from_rsa_pem(&Self::decode_key_base64(self.private_key_base64.clone())).unwrap()
  }

  pub fn load_public_key(&self) -> DecodingKey {
    DecodingKey::from_rsa_pem(&Self::decode_key_base64(self.public_key_base64.clone())).unwrap()
  }

  pub fn generate_jwt(
    &self,
    jti: Uuid,
    user_id: Uuid,
    token_type: TokenType,
    user_role: UserRole,
  ) -> String {
    let expiration_seconds: i64 = match token_type {
      TokenType::AccessToken => self.access_expiration_seconds,
      TokenType::RefreshToken => self.refresh_expiration_seconds,
    };

    let expiration: usize = if expiration_seconds == -1 {
      Self::DEFAULT_EXPIRATION
    } else {
      (get_current_timestamp() + expiration_seconds as u64) as usize
    };

    let claims = Claims {
      jti,
      token_type,
      sub: user_id,
      exp: expiration,
      role: user_role,
    };

    encode(
      &Header::new(self.algorithm),
      &claims,
      &self.load_private_key(),
    )
    .unwrap()
  }

  pub fn validate_jwt(&self, token: &str, token_type: TokenType) -> Option<Claims> {
    let mut relaxed_validation: Validation = Validation::new(self.algorithm);
    relaxed_validation.validate_exp = false;

    let token_data: TokenData<Claims> =
      match decode::<Claims>(token, &self.load_public_key(), &relaxed_validation) {
        Ok(data) if data.claims.token_type == token_type => data,
        Ok(_) => {
          tracing::warn!("Token type mismatch, should be {:?}", token_type);
          return None;
        }
        Err(e) => {
          tracing::warn!("Failed to decode JWT with relaxed validation: {:?}", e);
          return None;
        }
      };

    let mut hardened_validation = Validation::new(self.algorithm);
    hardened_validation.validate_exp = token_data.claims.exp != Self::DEFAULT_EXPIRATION;

    match decode::<Claims>(token, &self.load_public_key(), &hardened_validation) {
      Ok(data) => Some(data.claims),
      Err(e) => {
        tracing::warn!(
          "Failed to decode {:?} JWT: {:?}",
          token_data.claims.token_type,
          e
        );
        None
      }
    }
  }
}
