/*
backend/crates/repository/src/notify_config_repository/slack.rs
Slackの固有設定を司るリポジトリ
*/

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use identity::GroupId;

// 自クレート
use crate::RepositoryError;

/// Slackは仮実装のプレースホルダー
/// Discord実装完了後のMVP内拡張フェーズで本格的なフィールド構成に合わせて見直す
#[derive(Debug, Clone)]
pub struct NotifySlackConfigRow {
  pub webhook_url_ciphertext: Option<String>,
  pub mention_enabled: bool,
  pub mention_targets: Vec<String>,
}

/// notify_slack_configsテーブルのリポジトリ型
#[async_trait]
pub trait NotifySlackConfigRepository: Send + Sync {
  async fn find_by_group_id(
    &self,
    group_id: GroupId,
  ) -> Result<Option<NotifySlackConfigRow>, RepositoryError>;
  async fn upsert(
    &self,
    group_id: GroupId,
    row: &NotifySlackConfigRow,
  ) -> Result<(), RepositoryError>;
}
