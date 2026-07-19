/*
backend/crates/earnings/src/evaluation.rs
決算評価の列挙型とスクレイピング対象サイト列挙型を定義
*/

// 外部クレート
use serde::{Deserialize, Serialize};

/// 決算評価列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "earnings_evaluation", rename_all = "UPPERCASE")]
pub enum EarningsEvaluation {
  Positive,
  Neutral,
  Negative,
  Unrated,
}

/// ソースサイト列挙型
// 新しいスクレイピング対象サイトを追加する場合はここにvariantを追加する
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "earnings_source", rename_all = "lowercase")]
pub enum EarningsSource {
  Kabuyoho, // 株予報Pro
}
