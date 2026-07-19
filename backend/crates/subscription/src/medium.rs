/*
backend/crates/subscription/src/medium.rs
送信媒体の定義
*/

// 外部クレート
use serde::{Deserialize, Serialize};

/// 送信媒体の列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notify_medium", rename_all = "lowercase")]
pub enum NotifyMedium {
  Discord,
  Slack,
}
