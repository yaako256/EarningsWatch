/*
backend/crates/app/src/filter/update_filter.rs
フィルタ更新のユースケース
*/

// 内部ライブラリ
use earnings::normalize_ticker;
use identity::{FilterId, GroupId, UserId};
use repository::{NotifyFilterRepository, NotifyGroupRepository};
use subscription::NotifyFilter;

// 自クレート
use crate::AppError;

pub async fn update_filter(
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  requester_user_id: UserId,
  group_id: GroupId,
  filter_id: FilterId,
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

  let mut filter = filter_repo
    .find_by_id(filter_id)
    .await?
    .ok_or(AppError::NotFound)?;
  if filter.group_id != group_id {
    return Err(AppError::NotFound);
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

  filter.ticker = ticker;
  filter.company_name = company_name.trim().to_string();
  filter.notes = notes;

  filter_repo.update(&filter).await?;
  Ok(filter)
}
