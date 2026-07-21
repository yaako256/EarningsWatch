/*
backend/crates/api/src/state.rs
アプリの共有状態を定義
*/

// 外部クレート
use std::sync::Arc;

// 内部ライブラリ
use repository::{
  NotifyDiscordConfigRepository, NotifyFilterRepository, NotifyGroupRepository,
  NotifySlackConfigRepository, RefreshTokenRepository, UserRepository,
};

/// axumのRouterへ`.with_state()`で渡す共有状態
#[derive(Clone)]
pub struct AppState {
  pub user_repository: Arc<dyn UserRepository>,
  pub refresh_token_repository: Arc<dyn RefreshTokenRepository>,
  pub notify_group_repository: Arc<dyn NotifyGroupRepository>,
  pub notify_discord_config_repository: Arc<dyn NotifyDiscordConfigRepository>,
  pub notify_slack_config_repository: Arc<dyn NotifySlackConfigRepository>,
  pub notify_filter_repository: Arc<dyn NotifyFilterRepository>,
  pub unit_of_work: Arc<dyn repository::UnitOfWork>,
  pub jwt_secret: String,
  pub access_token_ttl_minutes: i64,
  pub refresh_token_ttl_days: i64,
  pub cookie_secure: bool,
  pub webhook_enc_key: Vec<u8>, // base64デコード済みの生バイト列
}
