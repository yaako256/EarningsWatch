/*
backend/crates/auth/src/token.rs
アクセストークンの型定義など
*/
use chrono::Utc;
use identity::UserId;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::Role;

/// アクセストークン(JWT)のクレーム。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
  pub sub: String, // UserId(UUID文字列)
  pub role: Role,
  pub iat: usize,
  pub exp: usize,
}

pub fn issue_access_token(
  user_id: UserId,
  role: Role,
  ttl_minutes: i64,
  secret: &str,
) -> Result<String, TokenError> {
  let now = Utc::now();
  let claims = TokenClaims {
    sub: user_id.as_uuid().to_string(),
    role,
    iat: now.timestamp() as usize,
    exp: (now + chrono::Duration::minutes(ttl_minutes)).timestamp() as usize,
  };

  encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
  )
  .map_err(|_| TokenError::EncodeFailed)
}

pub fn verify_access_token(token: &str, secret: &str) -> Result<TokenClaims, TokenError> {
  decode::<TokenClaims>(
    token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &Validation::default(),
  )
  .map(|data| data.claims)
  .map_err(|_| TokenError::InvalidOrExpired)
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
  #[error("トークンの生成に失敗しました")]
  EncodeFailed,
  #[error("トークンが無効または期限切れです")]
  InvalidOrExpired,
}
