/*
backend/crates/notifier/src/discord/config.rs
Discord固有の送信設定
*/

// 外部クレート
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use crypto::{Encrypted, WebhookUrlTag};
use identity::GroupId;

// 自クレート
use crate::discord::embed_color::EmbedColor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
  pub group_id: GroupId,
  pub webhook_url: Option<Encrypted<WebhookUrlTag>>,
  pub embed_color: Option<EmbedColor>,
  pub mention_enabled: bool,
  pub mention_targets: Vec<String>, // MentionTarget::parseで都度パースして使う
}
