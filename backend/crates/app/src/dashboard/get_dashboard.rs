/*
backend/crates/app/src/dashboard/get_dashboard.rs
ダッシュボードのユースケース
*/

// 内部ライブラリ
use config::DashboardSettings;
use identity::UserId;
use repository::{
  NotifyDiscordConfigRepository, NotifyFilterRepository, NotifyGroupRepository,
  NotifyHistoryRepository, NotifySlackConfigRepository,
};
use subscription::{NotifyHistoryEntry, NotifyMedium, NotifyStatus};

// 自クレート
use crate::AppError;

pub struct DashboardData {
  pub group_count: u32,
  pub filter_count: u32,
  pub unique_ticker_count: u32,
  pub unique_company_name_count: u32,
  pub discord_group_count: u32,
  pub slack_group_count: u32,
  pub paused_group_count: u32,
  pub webhook_missing_count: u32,
  pub recent_sent: Vec<NotifyHistoryEntry>,
  pub recent_failed: Vec<NotifyHistoryEntry>,
}

#[allow(clippy::too_many_arguments)]
pub async fn get_dashboard(
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  discord_config_repo: &dyn NotifyDiscordConfigRepository,
  slack_config_repo: &dyn NotifySlackConfigRepository,
  history_repo: &dyn NotifyHistoryRepository,
  settings: &DashboardSettings,
  user_id: UserId,
) -> Result<DashboardData, AppError> {
  let groups = group_repo.list_by_user_id(user_id).await?;

  let group_count = groups.len() as u32;
  let paused_group_count = groups.iter().filter(|g| g.paused_at.is_some()).count() as u32;
  let discord_group_count = groups
    .iter()
    .filter(|g| g.medium == NotifyMedium::Discord)
    .count() as u32;
  let slack_group_count = groups
    .iter()
    .filter(|g| g.medium == NotifyMedium::Slack)
    .count() as u32;

  // webhook未設定のグループ数(本書7.5節: N+1で判定、個人・家族・友人向け規模のため許容)
  let mut webhook_missing_count = 0u32;
  for group in &groups {
    let missing = match group.medium {
      NotifyMedium::Discord => discord_config_repo
        .find_by_group_id(group.id)
        .await?
        .map(|c| c.webhook_url_ciphertext.is_none())
        .unwrap_or(true),
      NotifyMedium::Slack => slack_config_repo
        .find_by_group_id(group.id)
        .await?
        .map(|c| c.webhook_url_ciphertext.is_none())
        .unwrap_or(true),
    };
    if missing {
      webhook_missing_count += 1;
    }
  }

  let filter_breakdown = filter_repo.count_breakdown_by_user(user_id).await?;

  let recent_sent = fetch_recent_hybrid(
    history_repo,
    user_id,
    NotifyStatus::Sent,
    settings.recent_sent_days,
    settings.recent_sent_min_count,
  )
  .await?;

  let recent_failed = fetch_recent_hybrid(
    history_repo,
    user_id,
    NotifyStatus::Failed,
    settings.recent_failed_days,
    settings.recent_failed_min_count,
  )
  .await?;

  Ok(DashboardData {
    group_count,
    filter_count: filter_breakdown.total,
    unique_ticker_count: filter_breakdown.unique_ticker_count,
    unique_company_name_count: filter_breakdown.unique_company_name_count,
    discord_group_count,
    slack_group_count,
    paused_group_count,
    webhook_missing_count,
    recent_sent,
    recent_failed,
  })
}

/// 本書3.2節のハイブリッド抽出ルール: 直近D日間の全件、それがM件未満ならD日間を無視してM件。
async fn fetch_recent_hybrid(
  history_repo: &dyn NotifyHistoryRepository,
  user_id: UserId,
  status: NotifyStatus,
  days: i64,
  min_count: u32,
) -> Result<Vec<NotifyHistoryEntry>, AppError> {
  let since = chrono::Utc::now() - chrono::Duration::days(days);
  let by_days = history_repo
    .list_recent_by_user_since(user_id, status, since)
    .await?;

  if by_days.len() as u32 >= min_count {
    Ok(by_days)
  } else {
    Ok(
      history_repo
        .list_recent_by_user_top_n(user_id, status, min_count)
        .await?,
    )
  }
}
