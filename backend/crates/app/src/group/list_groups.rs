/*
backend/crates/app/src/group/list_groups.rs
グループ一覧のユースケース
*/
use identity::UserId;
use repository::NotifyGroupRepository;
use subscription::NotifyGroup;

use crate::AppError;

pub async fn list_groups(
  group_repo: &dyn NotifyGroupRepository,
  user_id: UserId,
) -> Result<Vec<NotifyGroup>, AppError> {
  Ok(group_repo.list_by_user_id(user_id).await?)
}
