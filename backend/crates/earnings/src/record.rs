/*
backend/crates/earnings/src/record.rs
決算情報の構造体定義
*/

// 外部クレート
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 自クレート
use crate::{EarningsEvaluation, EarningsSource};

// ===== スクレイピング直後(DB保存前)の決算情報 =====
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Earnings {
  pub ticker: String,
  pub company_name: String,
  pub published_at: DateTime<Utc>,
  pub title: String,
  pub url: String,
  pub summary: String,
  pub evaluation: EarningsEvaluation,
}

// ===== スクレイピング結果全体(サイト共通、Python連携の受け渡し単位) =====
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredEarningsReport {
  pub schema_version: u32,
  pub source: EarningsSource,
  pub fetched_at: DateTime<Utc>,
  pub items: Vec<Earnings>,
}

// ===== DB保存後の決算情報(earningsテーブルの1行に対応) =====
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsRecord {
  pub id: i64,
  pub ticker: String,
  pub company_name: String,
  pub published_at: DateTime<Utc>,
  pub title: String,
  pub url: String,
  pub summary: String,
  pub evaluation: EarningsEvaluation,
  pub fingerprint: String,
  pub source: EarningsSource,
}
