/*
backend/crates/app/src/earnings/earnings_summary.rs
決算サマリーのユースケース
*/

// 外部クレート
use chrono::{DateTime, NaiveDate, Utc};

// 内部ライブラリ
use repository::EarningsRepository;

// 自クレート
use crate::AppError;

pub async fn earnings_summary(
  earnings_repo: &dyn EarningsRepository,
  from: Option<DateTime<Utc>>,
  to: Option<DateTime<Utc>>,
) -> Result<Vec<(NaiveDate, i64)>, AppError> {
  Ok(earnings_repo.summary_daily_counts_jst(from, to).await?)
}
