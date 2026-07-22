/*
backend/crates/app/src/admin/user_summary.rs
ユーザごとの軽い情報を確認するユースケース
*/

// 内部ライブラリ
use identity::UserId;
use repository::{NotifyFilterRepository, NotifyGroupRepository};
use subscription::NotifyMedium;

// 自クレート
use crate::AppError;

pub struct UserSummaryData {
  pub group_count: u32,
  pub filter_count: u32,
  pub discord_group_count: u32,
  pub slack_group_count: u32,
}

pub async fn user_summary(
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  user_id: UserId,
) -> Result<UserSummaryData, AppError> {
  let groups = group_repo.list_by_user_id(user_id).await?;
  let breakdown = filter_repo.count_breakdown_by_user(user_id).await?;

  Ok(UserSummaryData {
    group_count: groups.len() as u32,
    filter_count: breakdown.total,
    discord_group_count: groups
      .iter()
      .filter(|g| g.medium == NotifyMedium::Discord)
      .count() as u32,
    slack_group_count: groups
      .iter()
      .filter(|g| g.medium == NotifyMedium::Slack)
      .count() as u32,
  })
}
