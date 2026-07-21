/*
backend/crates/app/src/filter/list_filters.rs
フィルタ一覧のユースケース
*/

// 内部ライブラリ
use identity::{GroupId, UserId};
use repository::{NotifyFilterRepository, NotifyGroupRepository};
use subscription::NotifyFilter;

// 自クレート
use crate::AppError;

pub async fn list_filters(
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  requester_user_id: UserId,
  group_id: GroupId,
) -> Result<Vec<NotifyFilter>, AppError> {
  let group = group_repo
    .find_by_id(group_id)
    .await?
    .ok_or(AppError::NotFound)?;
  if group.user_id != requester_user_id {
    return Err(AppError::Forbidden);
  }

  Ok(filter_repo.list_by_group_id(group_id).await?)
}
