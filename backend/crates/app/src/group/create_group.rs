/*
backend/crates/app/src/group/create_group.rs
新規グループを作成するユースケース
*/

// 外部クレート
use chrono::Utc;

// 内部ライブラリ
use identity::{GroupId, UserId};
use repository::{NotifyDiscordConfigRow, NotifySlackConfigRow, RepositoryScope, UnitOfWork};
use subscription::{NotifyGroup, NotifyMedium};

// 自クレート
use crate::AppError;

// グループ名の最大文字数
const GROUP_NAME_MAX_LEN: usize = 30;

pub async fn create_group<U: UnitOfWork>(
  uow: &U,
  user_id: UserId,
  name: String,
  medium: NotifyMedium,
) -> Result<NotifyGroup, AppError> {
  let name = name.trim().to_string();
  if name.is_empty() || name.chars().count() > GROUP_NAME_MAX_LEN {
    return Err(AppError::InvalidInput(format!(
      "グループ名は1〜{GROUP_NAME_MAX_LEN}文字で入力してください"
    )));
  }

  let now = Utc::now();
  let group = NotifyGroup {
    id: GroupId::new(),
    user_id,
    name,
    medium,
    paused_at: None,
    created_at: now,
    updated_at: now,
  };

  let group_for_closure = group.clone();

  uow
    .execute(move |scope: &mut dyn RepositoryScope| {
      Box::pin(async move {
        scope
          .notify_group_repository()
          .insert(&group_for_closure)
          .await?;

        scope
          .notify_discord_config_repository()
          .upsert(
            group_for_closure.id,
            &NotifyDiscordConfigRow {
              webhook_url_ciphertext: None,
              embed_color: None,
              mention_enabled: false,
              mention_targets: vec![],
            },
          )
          .await?;

        scope
          .notify_slack_config_repository()
          .upsert(
            group_for_closure.id,
            &NotifySlackConfigRow {
              webhook_url_ciphertext: None,
              mention_enabled: false,
              mention_targets: vec![],
            },
          )
          .await?;

        Ok(())
      })
    })
    .await?;

  Ok(group)
}
