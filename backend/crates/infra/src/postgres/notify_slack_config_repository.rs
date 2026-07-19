/*
backend/crates/infra/src/postgres/notify_slack_config_repository.rs
Slack通知固有設定テーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;

// 内部ライブラリ
use identity::GroupId;
use repository::{NotifySlackConfigRepository, NotifySlackConfigRow, RepositoryError};

// 自クレート
use crate::error_mapping::map_error;

pub struct PgNotifySlackConfigRepository {
  pool: PgPool,
}

impl PgNotifySlackConfigRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl NotifySlackConfigRepository for PgNotifySlackConfigRepository {
  async fn find_by_group_id(
    &self,
    group_id: GroupId,
  ) -> Result<Option<NotifySlackConfigRow>, RepositoryError> {
    let row = sqlx::query!(
      "SELECT webhook_url, mention_enabled, mention_targets FROM notify_slack_configs WHERE group_id = $1",
      group_id.as_uuid()
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(row.map(|r| NotifySlackConfigRow {
      webhook_url_ciphertext: r.webhook_url,
      mention_enabled: r.mention_enabled,
      mention_targets: r.mention_targets,
    }))
  }

  async fn upsert(
    &self,
    group_id: GroupId,
    row: &NotifySlackConfigRow,
  ) -> Result<(), RepositoryError> {
    sqlx::query!(
      r#"
      INSERT INTO notify_slack_configs (group_id, webhook_url, mention_enabled, mention_targets)
      VALUES ($1, $2, $3, $4)
      ON CONFLICT (group_id) DO UPDATE
      SET webhook_url = EXCLUDED.webhook_url,
        mention_enabled = EXCLUDED.mention_enabled,
        mention_targets = EXCLUDED.mention_targets
      "#,
      group_id.as_uuid(),
      row.webhook_url_ciphertext,
      row.mention_enabled,
      &row.mention_targets
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }
}
