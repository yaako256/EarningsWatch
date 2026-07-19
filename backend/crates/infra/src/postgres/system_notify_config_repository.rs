/*
backend/crates/infra/src/postgres/system_notify_config_repository.rs
システム通知設定テーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;

// 内部ライブラリ
use crypto::Encrypted;
use repository::{RepositoryError, SystemNotifyConfigRepository};
use subscription::{NotifyMedium, SystemNotifyConfig};

// 自クレート
use crate::error_mapping::map_error;

pub struct PgSystemNotifyConfigRepository {
  pool: PgPool,
}

impl PgSystemNotifyConfigRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl SystemNotifyConfigRepository for PgSystemNotifyConfigRepository {
  async fn get(&self) -> Result<Option<SystemNotifyConfig>, RepositoryError> {
    let row = sqlx::query!(
      r#"
      SELECT medium as "medium: NotifyMedium", webhook_url, mention_enabled, mention_targets, updated_at
      FROM system_notify_config WHERE id = TRUE
      "#
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(row.map(|r| SystemNotifyConfig {
      medium: r.medium,
      webhook_url: r.webhook_url.map(Encrypted::from_ciphertext),
      mention_enabled: r.mention_enabled,
      mention_targets: r.mention_targets,
      updated_at: r.updated_at,
    }))
  }

  async fn upsert(&self, config: &SystemNotifyConfig) -> Result<(), RepositoryError> {
    sqlx::query!(
      r#"
      INSERT INTO system_notify_config (id, medium, webhook_url, mention_enabled, mention_targets, updated_at)
      VALUES (TRUE, $1, $2, $3, $4, $5)
      ON CONFLICT (id) DO UPDATE
      SET medium = EXCLUDED.medium,
          webhook_url = EXCLUDED.webhook_url,
          mention_enabled = EXCLUDED.mention_enabled,
          mention_targets = EXCLUDED.mention_targets,
          updated_at = EXCLUDED.updated_at
      "#,
      config.medium as NotifyMedium,
      config.webhook_url.as_ref().map(|e| e.as_str().to_string()),
      config.mention_enabled,
      &config.mention_targets,
      config.updated_at
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }
}
