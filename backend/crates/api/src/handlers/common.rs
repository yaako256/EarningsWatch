/*
backend/crates/api/src/handlers/common.rs
複数のハンドラで使う共通なものを定義
*/

// 外部クレート
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use identity::{FilterId, GroupId};
use subscription::{NotifyFilter, NotifyGroup, NotifyMedium};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupResponse {
  pub id: GroupId,
  pub name: String,
  pub medium: NotifyMedium,
  pub paused_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<NotifyGroup> for GroupResponse {
  fn from(g: NotifyGroup) -> Self {
    Self {
      id: g.id,
      name: g.name,
      medium: g.medium,
      paused_at: g.paused_at,
      created_at: g.created_at,
      updated_at: g.updated_at,
    }
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterResponse {
  pub id: FilterId,
  pub group_id: GroupId,
  pub ticker: String,
  pub company_name: String,
  pub notes: Option<String>,
  pub enabled: bool,
}

impl From<NotifyFilter> for FilterResponse {
  fn from(f: NotifyFilter) -> Self {
    Self {
      id: f.id,
      group_id: f.group_id,
      ticker: f.ticker,
      company_name: f.company_name,
      notes: f.notes,
      enabled: f.enabled,
    }
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRef {
  pub id: GroupId,
  pub name: String,
}

// 決算情報・フィルタのエクスポートで共有するformat指定。
// MVPではxlsxのみ対応(CSV対応は将来拡張)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportFormat {
  Xlsx,
}
