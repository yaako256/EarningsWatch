/*
backend/crates/app/src/group/pause_resume_group.rs
グループを一時停止/停止解除するユースケース
*/
use identity::{GroupId, UserId};
use repository::NotifyGroupRepository;
use subscription::NotifyGroup;

use crate::AppError;

async fn set_paused(
  group_repo: &dyn NotifyGroupRepository,
  requester_user_id: UserId,
  group_id: GroupId,
  paused: bool,
) -> Result<NotifyGroup, AppError> {
  let mut group = group_repo
    .find_by_id(group_id)
    .await?
    .ok_or(AppError::NotFound)?;
  if group.user_id != requester_user_id {
    return Err(AppError::Forbidden);
  }

  group.paused_at = if paused {
    Some(chrono::Utc::now())
  } else {
    None
  };
  group.updated_at = chrono::Utc::now();

  group_repo.update(&group).await?;
  Ok(group)
}

pub async fn pause_group(
  group_repo: &dyn NotifyGroupRepository,
  requester_user_id: UserId,
  group_id: GroupId,
) -> Result<NotifyGroup, AppError> {
  set_paused(group_repo, requester_user_id, group_id, true).await
}

pub async fn resume_group(
  group_repo: &dyn NotifyGroupRepository,
  requester_user_id: UserId,
  group_id: GroupId,
) -> Result<NotifyGroup, AppError> {
  set_paused(group_repo, requester_user_id, group_id, false).await
}
