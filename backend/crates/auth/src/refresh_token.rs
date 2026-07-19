/*
backend/crates/auth/src/refresh_token.rs
RefreshToken構造体の定義
*/

// 外部クレート
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use identity::{RefreshTokenId, UserId};

/// リフレッシュトークン型
// DB: refresh_tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
  pub id: RefreshTokenId,
  pub user_id: UserId,
  pub token_hash: String,
  pub user_agent: Option<String>,
  pub expires_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  pub revoked_at: Option<DateTime<Utc>>,
}

impl RefreshToken {
  pub fn is_revoked(&self) -> bool {
    self.revoked_at.is_some()
  }

  pub fn is_expired_at(&self, now: DateTime<Utc>) -> bool {
    self.expires_at <= now
  }
}
