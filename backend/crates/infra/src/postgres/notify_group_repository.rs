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
use crate::error_mapping::{map_conflict_error, map_error};

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
    let row = sqlx::query_as!(
      NotifyGroupRow,
      r#"
      SELECT id, user_id, name, medium as "medium: NotifyMedium",
              paused_at, created_at, updated_at
      FROM notify_groups WHERE id = $1
      "#,
      id.as_uuid()
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(row.map(NotifyGroup::from))
  }

  async fn list_by_user_id(&self, user_id: UserId) -> Result<Vec<NotifyGroup>, RepositoryError> {
    let rows = sqlx::query_as!(
      NotifyGroupRow,
      r#"
      SELECT id, user_id, name, medium as "medium: NotifyMedium",
              paused_at, created_at, updated_at
      FROM notify_groups WHERE user_id = $1
      ORDER BY created_at ASC
      "#,
      user_id.as_uuid()
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(rows.into_iter().map(NotifyGroup::from).collect())
  }

  async fn list_all(&self) -> Result<Vec<NotifyGroup>, RepositoryError> {
    let rows = sqlx::query_as!(
      NotifyGroupRow,
      r#"
      SELECT id, user_id, name, medium as "medium: NotifyMedium", paused_at, created_at, updated_at
      FROM notify_groups
      ORDER BY created_at ASC
      "#
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(rows.into_iter().map(NotifyGroup::from).collect())
  }

  async fn insert(&self, group: &NotifyGroup) -> Result<(), RepositoryError> {
    sqlx::query!(
      r#"
      INSERT INTO notify_groups (id, user_id, name, medium, paused_at, created_at, updated_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7)
      "#,
      group.id.as_uuid(),
      group.user_id.as_uuid(),
      group.name,
      group.medium as NotifyMedium,
      group.paused_at,
      group.created_at,
      group.updated_at
    )
    .execute(&self.pool)
    .await
    .map_err(map_conflict_error)?;

    Ok(())
  }

  async fn update(&self, group: &NotifyGroup) -> Result<(), RepositoryError> {
    sqlx::query!(
      r#"
      UPDATE notify_groups
      SET name = $2, medium = $3, paused_at = $4, updated_at = $5
      WHERE id = $1
      "#,
      group.id.as_uuid(),
      group.name,
      group.medium as NotifyMedium,
      group.paused_at,
      group.updated_at
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }

  async fn delete(&self, id: GroupId) -> Result<(), RepositoryError> {
    sqlx::query!("DELETE FROM notify_groups WHERE id = $1", id.as_uuid())
      .execute(&self.pool)
      .await
      .map_err(map_error)?;

    Ok(())
  }
}
