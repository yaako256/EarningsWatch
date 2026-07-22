/*
backend/crates/api/src/state.rs
アプリの共有状態を定義
*/

// 標準ライブラリ
use std::sync::Arc;

// 内部ライブラリ
use config::{DashboardSettings, ImportSettings};
use repository::{
  EarningsRepository, LogRepository, NotifyDiscordConfigRepository, NotifyFilterRepository,
  NotifyGroupRepository, NotifyHistoryRepository, NotifyQueueRepository,
  NotifySlackConfigRepository, PageRepository, RefreshTokenRepository,
  SystemNotifyConfigRepository, SystemRunRepository, UserRepository,
};

/// axumのRouterへ`.with_state()`で渡す共有状態
#[derive(Clone)]
pub struct AppState {
  // リポジトリ系
  pub user_repository: Arc<dyn UserRepository>,
  pub refresh_token_repository: Arc<dyn RefreshTokenRepository>,
  pub earnings_repository: Arc<dyn EarningsRepository>,
  pub notify_group_repository: Arc<dyn NotifyGroupRepository>,
  pub notify_discord_config_repository: Arc<dyn NotifyDiscordConfigRepository>,
  pub notify_slack_config_repository: Arc<dyn NotifySlackConfigRepository>,
  pub notify_filter_repository: Arc<dyn NotifyFilterRepository>,
  pub notify_history_repository: Arc<dyn NotifyHistoryRepository>,
  pub notify_queue_repository: Arc<dyn NotifyQueueRepository>,
  pub log_repository: Arc<dyn LogRepository>,
  pub system_notify_config_repository: Arc<dyn SystemNotifyConfigRepository>,
  pub system_run_repository: Arc<dyn SystemRunRepository>,
  pub page_repository: Arc<dyn PageRepository>,
  pub unit_of_work: Arc<dyn repository::UnitOfWork>,
  // 設定系
  pub jwt_secret: String,
  pub access_token_ttl_minutes: i64,
  pub refresh_token_ttl_days: i64,
  pub cookie_secure: bool,
  pub webhook_enc_key: Vec<u8>, // base64デコード済みの生バイト列
  pub import_settings: ImportSettings,
  pub dashboard_settings: DashboardSettings,
}
