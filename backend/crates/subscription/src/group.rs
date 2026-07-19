/*
backend/crates/subscription/src/filter.rs
グループ単位での設定項目の型定義
*/

// 外部クレート
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use identity::{GroupId, UserId};

// 自クレート
use crate::NotifyMedium;

/// グループ単位の設定構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyGroup {
  pub id: GroupId,
  pub user_id: UserId,
  pub name: String,
  pub medium: NotifyMedium,
  pub paused_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl NotifyGroup {
  pub fn is_paused(&self) -> bool {
    self.paused_at.is_some()
  }
}
