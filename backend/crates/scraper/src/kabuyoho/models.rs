/*
backend/crates/scraper/src/kabuyoho/models.rs
株予報Pro用の型定義
*/

// 外部クレート
use serde::{Deserialize, Serialize};

/// 一覧ページから取得する、新規/既知判定用の生データ(kabuyoho専用のフィールド名)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KabuyohoListItem {
  pub fingerprint_title: String,
  pub fingerprint_summary: String,
  pub fingerprint_evaluation: String,
  pub url: String,
}

#[derive(Deserialize)]
pub struct KabuyohoListOutput {
  pub items: Vec<KabuyohoListItem>,
}

#[derive(Deserialize)]
pub struct KabuyohoDetailOutput {
  pub ticker: String,
  pub company_name: String,
  pub published_at: chrono::DateTime<chrono::Utc>,
  pub title: String,
  pub url: String,
  pub summary: String,
  // evaluationは含まれない(一覧段階のevaluationをRust側で埋める)
}
