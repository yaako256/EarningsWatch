/*
backend/crates/app/src/group/update_group.rs
グループを更新する系のユースケース群
*/
use identity::{GroupId, UserId};
use repository::NotifyGroupRepository;
use subscription::{NotifyGroup, NotifyMedium};

use crate::AppError;

// グループ名の最大文字数
const GROUP_NAME_MAX_LEN: usize = 30;

pub async fn update_group(
  group_repo: &dyn NotifyGroupRepository,
  requester_user_id: UserId,
  group_id: GroupId,
  name: String,
  medium: NotifyMedium,
) -> Result<NotifyGroup, AppError> {
  let name = name.trim().to_string();
  if name.is_empty() || name.chars().count() > GROUP_NAME_MAX_LEN {
    return Err(AppError::InvalidInput(format!(
      "グループ名は1〜{GROUP_NAME_MAX_LEN}文字で入力してください"
    )));
  }

  let mut group = group_repo
    .find_by_id(group_id)
    .await?
    .ok_or(AppError::NotFound)?;
  if group.user_id != requester_user_id {
    return Err(AppError::Forbidden);
  }

  group.name = name;
  group.medium = medium;
  group.updated_at = chrono::Utc::now();

  group_repo.update(&group).await?;
  Ok(group)
}
