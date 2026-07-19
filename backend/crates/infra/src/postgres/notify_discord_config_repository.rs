/*
backend/crates/infra/src/postgres/notify_discord_config_repository.rs
Discord固有設定テーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;

// 内部ライブラリ
use identity::GroupId;
use repository::{NotifyDiscordConfigRepository, NotifyDiscordConfigRow, RepositoryError};

// 自クレート
use crate::error_mapping::map_error;

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
  ) -> Result<Option<NotifyDiscordConfigRow>, RepositoryError> {
    let row = sqlx::query!(
      r#"
      SELECT webhook_url, embed_color, mention_enabled, mention_targets
      FROM notify_discord_configs WHERE group_id = $1
      "#,
      group_id.as_uuid()
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(row.map(|r| NotifyDiscordConfigRow {
      webhook_url_ciphertext: r.webhook_url,
      embed_color: r.embed_color,
      mention_enabled: r.mention_enabled,
      mention_targets: r.mention_targets,
    }))
  }

  async fn upsert(
    &self,
    group_id: GroupId,
    row: &NotifyDiscordConfigRow,
  ) -> Result<(), RepositoryError> {
    sqlx::query!(
      r#"
      INSERT INTO notify_discord_configs (group_id, webhook_url, embed_color, mention_enabled, mention_targets)
      VALUES ($1, $2, $3, $4, $5)
      ON CONFLICT (group_id) DO UPDATE
      SET webhook_url = EXCLUDED.webhook_url,
          embed_color = EXCLUDED.embed_color,
          mention_enabled = EXCLUDED.mention_enabled,
          mention_targets = EXCLUDED.mention_targets
      "#,
      group_id.as_uuid(),
      row.webhook_url_ciphertext,
      row.embed_color,
      row.mention_enabled,
      &row.mention_targets
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }
}
