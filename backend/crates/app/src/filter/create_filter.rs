/*
backend/crates/app/src/filter/create_filter.rs
フィルタを追加するユースケース
*/

// 外部クレート
use chrono::Utc;

// 内部ライブラリ
use earnings::normalize_ticker;
use identity::{FilterId, GroupId, UserId};
use repository::{NotifyFilterRepository, NotifyGroupRepository};
use subscription::NotifyFilter;

// 自クレート
use crate::AppError;

pub async fn create_filter(
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  requester_user_id: UserId,
  group_id: GroupId,
  ticker: String,
  company_name: String,
  notes: Option<String>,
) -> Result<NotifyFilter, AppError> {
  let group = group_repo
    .find_by_id(group_id)
    .await?
    .ok_or(AppError::NotFound)?;
  if group.user_id != requester_user_id {
    return Err(AppError::Forbidden);
  }

  let ticker = normalize_ticker(&ticker);
  if ticker.is_empty() {
    return Err(AppError::InvalidInput(
      "tickerを入力してください".to_string(),
    ));
  }
  if company_name.trim().is_empty() {
    return Err(AppError::InvalidInput(
      "company_nameを入力してください".to_string(),
    ));
  }

  let filter = NotifyFilter {
    id: FilterId::new(),
    group_id,
    ticker,
    company_name: company_name.trim().to_string(),
    notes,
    enabled: true,
    created_at: Utc::now(),
  };

  filter_repo.insert(&filter).await?;
  Ok(filter)
}
