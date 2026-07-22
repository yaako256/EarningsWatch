// crates/notifier/src/slack_config.rs
// 仮実装。Discord実装完了後のMVP内拡張フェーズで再定義する(引き継ぎメモ「Slack関連」参照)。
use crypto::{Encrypted, WebhookUrlTag};
use identity::GroupId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
pub struct SlackConfig {
  pub group_id: GroupId,
  pub webhook_url: Option<Encrypted<WebhookUrlTag>>,
  pub mention_enabled: bool,
  pub mention_targets: Vec<String>,
}
