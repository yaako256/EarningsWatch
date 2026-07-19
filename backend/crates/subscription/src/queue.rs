/*
backend/crates/subscription/src/queue.rs
送信キューの型定義
*/

// 外部クレート
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use earnings::{EarningsEvaluation, EarningsSource};

/// 送信statusの列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notify_status", rename_all = "lowercase")]
pub enum NotifyStatus {
  Ready,
  Sent,
  Failed,
}

/// 送信キュー構造体
// is_monitor_marker=trueの行(健全性チェック用マーカー)はこの型では表現しない。
// 決算データ行(is_monitor_marker=false)のみをこの型で表現する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyQueueEntry {
  pub id: i64,
  pub fingerprint: String,
  pub source: EarningsSource,
  pub fetched_at: DateTime<Utc>,
  pub ticker: String,
  pub company_name: String,
  pub published_at: DateTime<Utc>,
  pub title: String,
  pub url: String,
  pub summary: String,
  pub evaluation: EarningsEvaluation,
  pub status: NotifyStatus,
}
