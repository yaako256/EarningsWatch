/*
backend/crates/subscription/src/history.rs
送信履歴を管理する型定義
*/

// 外部クレート
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use identity::GroupId;

// 自クレート
use crate::NotifyStatus;

/// 送信履歴の構造体
// group_idはON DELETE SET NULL化に伴いOption<GroupId>とする。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyHistoryEntry {
  pub id: i64,
  pub group_id: Option<GroupId>,
  pub fingerprint: String,
  pub sent_at: DateTime<Utc>,
  pub status: NotifyStatus,
}
