/*
backend/crates/scraper/src/debug/models.rs
サイト固有のフィールド構成
デバッグ用スクレイピングの型定義
*/

use serde::{Deserialize, Serialize};

/// 一覧ページから取得する、新規/既知判定用の生データ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugListItem {
  pub fingerprint_item_1: String,
  pub fingerprint_item_2: String,
  pub fingerprint_item_3: String,
  pub url: String,
}

#[derive(Deserialize)]
pub struct DebugListOutput {
  pub items: Vec<DebugListItem>,
}

#[derive(Deserialize)]
pub struct DebugDetailOutput {
  pub ticker: String,
  pub company_name: String,
  pub published_at: chrono::DateTime<chrono::Utc>,
  pub title: String,
  pub url: String,
  pub summary: String,
  pub evaluation: String,
}
