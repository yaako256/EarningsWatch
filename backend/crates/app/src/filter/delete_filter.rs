/*
backend/crates/app/src/filter/delete_filter.rs
フィルタを削除するユースケース
*/

// 内部ライブラリ
use identity::{FilterId, GroupId, UserId};
use repository::{NotifyFilterRepository, NotifyGroupRepository};

// 自クレート
use crate::AppError;

pub async fn delete_filter(
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  requester_user_id: UserId,
  group_id: GroupId,
  filter_id: FilterId,
) -> Result<(), AppError> {
  let group = group_repo
    .find_by_id(group_id)
    .await?
    .ok_or(AppError::NotFound)?;
  if group.user_id != requester_user_id {
    return Err(AppError::Forbidden);
  }

  let filter = filter_repo
    .find_by_id(filter_id)
    .await?
    .ok_or(AppError::NotFound)?;
  if filter.group_id != group_id {
    return Err(AppError::NotFound);
  }

  filter_repo.delete(filter_id).await?;
  Ok(())
}
