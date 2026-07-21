/*
backend/crates/infra/src/postgres/user_settings_repository.rs
ユーザ設定テーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;

// 内部ライブラリ
use identity::UserId;
use repository::{RepositoryResult, UserSettingsRepository};
use subscription::UserSettings;

// 自クレート
use crate::error_mapping::map_error;

pub struct PgUserSettingsRepository {
  pool: PgPool,
}

impl PgUserSettingsRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl UserSettingsRepository for PgUserSettingsRepository {
  async fn find_by_user_id(&self, user_id: UserId) -> RepositoryResult<Option<UserSettings>> {
    let row = sqlx::query!(
      "SELECT user_id, memo, updated_at FROM user_settings WHERE user_id = $1",
      user_id.as_uuid()
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(row.map(|r| UserSettings {
      user_id: UserId::from_uuid(r.user_id),
      memo: r.memo,
      updated_at: r.updated_at,
    }))
  }

  async fn upsert(&self, settings: &UserSettings) -> RepositoryResult<()> {
    sqlx::query!(
      r#"
      INSERT INTO user_settings (user_id, memo, updated_at)
      VALUES ($1, $2, $3)
      ON CONFLICT (user_id) DO UPDATE SET memo = EXCLUDED.memo, updated_at = EXCLUDED.updated_at
      "#,
      settings.user_id.as_uuid(),
      settings.memo,
      settings.updated_at
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }
}
