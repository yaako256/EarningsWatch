/*
backend/crates/repository/src/notify_config_repository/discord.rs
Discordの固有設定を司るリポジトリ
*/

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use identity::GroupId;

// 自クレート
use crate::RepositoryResult;

/// notify_discord_configsテーブルの列構成にそのまま対応するプリミティブ型のRow DTO
/// notifier::DiscordConfigへの変換(暗号文のEncrypted<T>ラップ、embed_colorのパース)はapp層で行う
/// (repositoryクレートはnotifierに依存しないため)
#[derive(Debug, Clone)]
pub struct NotifyDiscordConfigRow {
  // base64(nonce || ciphertext)、まだEncrypted<T>にラップしない生の値
  pub webhook_url_ciphertext: Option<String>,
  // "0x87EB87"形式の生の文字列、まだEmbedColorにパースしない
  pub embed_color: Option<String>,
  pub mention_enabled: bool,
  pub mention_targets: Vec<String>,
}

/// notify_discord_configsテーブルのリポジトリ型
#[async_trait]
pub trait NotifyDiscordConfigRepository: Send + Sync {
  async fn find_by_group_id(
    &self,
    group_id: GroupId,
  ) -> RepositoryResult<Option<NotifyDiscordConfigRow>>;
  async fn upsert(&self, group_id: GroupId, row: &NotifyDiscordConfigRow) -> RepositoryResult<()>;
}
