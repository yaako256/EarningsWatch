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
use super::queries::notify_slack_config;

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
    notify_slack_config::find_by_group_id(&self.pool, group_id).await
  }

  async fn upsert(
    &self,
    group_id: GroupId,
    row: &NotifySlackConfigRow,
  ) -> Result<(), RepositoryError> {
    notify_slack_config::upsert(&self.pool, group_id, row).await
  }
}
