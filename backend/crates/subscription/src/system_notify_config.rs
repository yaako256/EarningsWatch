/*
backend/crates/subscription/src/system_notify_config.rs
管理者用送信設定の型定義
*/

// 外部クレート
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use crypto::{Encrypted, SystemNotifyWebhookUrlTag};

// 自クレート
use crate::NotifyMedium;

/// 管理者用送信設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemNotifyConfig {
  pub medium: NotifyMedium,
  pub webhook_url: Option<Encrypted<SystemNotifyWebhookUrlTag>>,
  pub mention_enabled: bool,
  pub mention_targets: Vec<String>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}
