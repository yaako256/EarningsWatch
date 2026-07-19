/*
backend/crates/infra/src/postgres/notify_group_repository.rs
通知グループテーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// 内部ライブラリ
use identity::{GroupId, UserId};
use repository::{NotifyGroupRepository, RepositoryError};
use subscription::{NotifyGroup, NotifyMedium};

// 自クレート
use super::queries::notify_group;

pub struct PgNotifyGroupRepository {
  pool: PgPool,
}

impl PgNotifyGroupRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

struct NotifyGroupRow {
  id: Uuid,
  user_id: Uuid,
  name: String,
  medium: NotifyMedium,
  paused_at: Option<chrono::DateTime<chrono::Utc>>,
  created_at: chrono::DateTime<chrono::Utc>,
  updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<NotifyGroupRow> for NotifyGroup {
  fn from(row: NotifyGroupRow) -> Self {
    NotifyGroup {
      id: GroupId::from_uuid(row.id),
      user_id: UserId::from_uuid(row.user_id),
      name: row.name,
      medium: row.medium,
      paused_at: row.paused_at,
      created_at: row.created_at,
      updated_at: row.updated_at,
    }
  }
}

#[async_trait]
impl NotifyGroupRepository for PgNotifyGroupRepository {
  async fn find_by_id(&self, id: GroupId) -> Result<Option<NotifyGroup>, RepositoryError> {
    notify_group::find_by_id(&self.pool, id).await
  }

  async fn list_by_user_id(&self, user_id: UserId) -> Result<Vec<NotifyGroup>, RepositoryError> {
    notify_group::list_by_user_id(&self.pool, user_id).await
  }

  async fn list_all(&self) -> Result<Vec<NotifyGroup>, RepositoryError> {
    notify_group::list_all(&self.pool).await
  }

  async fn insert(&self, group: &NotifyGroup) -> Result<(), RepositoryError> {
    notify_group::insert(&self.pool, group).await
  }

  async fn update(&self, group: &NotifyGroup) -> Result<(), RepositoryError> {
    notify_group::update(&self.pool, group).await
  }

  async fn delete(&self, id: GroupId) -> Result<(), RepositoryError> {
    notify_group::delete(&self.pool, id).await
  }
}
