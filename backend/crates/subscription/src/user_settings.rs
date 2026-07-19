/*
backend/crates/subscription/src/user_settings.rs
ユーザ設定の型定義
*/

// 外部クレート
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use identity::UserId;

/// ユーザ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
  pub user_id: UserId,
  pub memo: Option<String>,
  pub updated_at: DateTime<Utc>,
}
