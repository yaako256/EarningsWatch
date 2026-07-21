/*
backend/crates/app/src/earnings/list_earnings.rs
決算情報リストのユースケース
*/

// 内部ライブラリ
use earnings::EarningsRecord;
use repository::{EarningsListFilter, EarningsRepository};

// 自クレート
use crate::AppError;

pub async fn list_earnings(
  earnings_repo: &dyn EarningsRepository,
  filter: EarningsListFilter,
  page: u32,
  per_page: u32,
) -> Result<(Vec<EarningsRecord>, i64), AppError> {
  Ok(earnings_repo.list_filtered(&filter, page, per_page).await?)
}
