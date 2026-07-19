/*
backend/crates/auth/src/user.rs
ユーザ構造体の定義
*/

// 外部クレート
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use identity::UserId;

// 自クレート
use crate::Role;

/// ユーザの構造体
// DB: users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
  pub id: UserId,
  pub username: String,
  pub password_hash: String,
  pub role: Role,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub disabled_at: Option<DateTime<Utc>>,
}

impl User {
  pub fn is_disabled(&self) -> bool {
    self.disabled_at.is_some()
  }
}
