/*
backend/crates/subscription/src/filter.rs
フィルターの設定項目の型定義
*/

// 外部クレート
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use identity::{FilterId, GroupId};

/// フィルター設定構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyFilter {
  pub id: FilterId,
  pub group_id: GroupId,
  pub ticker: String,
  pub company_name: String,
  pub notes: Option<String>,
  pub enabled: bool,
  pub created_at: DateTime<Utc>,
}
