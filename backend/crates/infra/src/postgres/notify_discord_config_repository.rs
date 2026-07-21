/*
backend/crates/infra/src/postgres/notify_discord_config_repository.rs
Discord固有設定テーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;

// 内部ライブラリ
use identity::GroupId;
use repository::{NotifyDiscordConfigRepository, NotifyDiscordConfigRow, RepositoryResult};

// 自クレート
use super::queries::notify_discord_config;

pub struct PgNotifyDiscordConfigRepository {
  pool: PgPool,
}

impl PgNotifyDiscordConfigRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl NotifyDiscordConfigRepository for PgNotifyDiscordConfigRepository {
  async fn find_by_group_id(
    &self,
    group_id: GroupId,
  ) -> RepositoryResult<Option<NotifyDiscordConfigRow>> {
    notify_discord_config::find_by_group_id(&self.pool, group_id).await
  }

  async fn upsert(&self, group_id: GroupId, row: &NotifyDiscordConfigRow) -> RepositoryResult<()> {
    notify_discord_config::upsert(&self.pool, group_id, row).await
  }
}
