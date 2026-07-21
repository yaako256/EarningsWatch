/*
backend/crates/app/src/filter/toggle_filter.rs
フィルタのトグル設定のユースケース
*/

// 内部ライブラリ
use identity::{FilterId, GroupId, UserId};
use repository::{NotifyFilterRepository, NotifyGroupRepository};

// 自クレート
use crate::AppError;

async fn set_enabled(
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  requester_user_id: UserId,
  group_id: GroupId,
  filter_id: FilterId,
  enabled: bool,
) -> Result<(), AppError> {
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

  filter.enabled = enabled;
  filter_repo.update(&filter).await?;
  Ok(())
}

pub async fn enable_filter(
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  requester_user_id: UserId,
  group_id: GroupId,
  filter_id: FilterId,
) -> Result<(), AppError> {
  set_enabled(
    group_repo,
    filter_repo,
    requester_user_id,
    group_id,
    filter_id,
    true,
  )
  .await
}

pub async fn disable_filter(
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  requester_user_id: UserId,
  group_id: GroupId,
  filter_id: FilterId,
) -> Result<(), AppError> {
  set_enabled(
    group_repo,
    filter_repo,
    requester_user_id,
    group_id,
    filter_id,
    false,
  )
  .await
}
