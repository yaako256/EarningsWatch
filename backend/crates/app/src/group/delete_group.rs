/*
backend/crates/app/src/group/delete_group.rs
グループを削除するユースケース
*/

// 内部ライブラリ
use identity::{GroupId, UserId};
use repository::NotifyGroupRepository;

// 自クレート
use crate::AppError;

pub async fn delete_group(
  group_repo: &dyn NotifyGroupRepository,
  requester_user_id: UserId,
  group_id: GroupId,
) -> Result<(), AppError> {
  let group = group_repo
    .find_by_id(group_id)
    .await?
    .ok_or(AppError::NotFound)?;
  if group.user_id != requester_user_id {
    return Err(AppError::Forbidden);
  }

  // notify_discord_configs/notify_slack_configs/notify_filtersはgroup_idにON DELETE CASCADEが
  // 張られている前提(01-db-schema.md 4章)のため、notify_groups行の削除のみでよい。
  group_repo.delete(group_id).await?;
  Ok(())
}
