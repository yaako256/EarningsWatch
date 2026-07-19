// backend/crates/infra/src/postgres/queries/notify_group.rs

use chrono::{DateTime, Utc};
use identity::{GroupId, UserId};
use repository::RepositoryError;
use sqlx::{Executor, Postgres};
use subscription::{NotifyGroup, NotifyMedium};
use uuid::Uuid;

use crate::error_mapping::{map_conflict_error, map_error};

struct NotifyGroupRow {
  id: Uuid,
  user_id: Uuid,
  name: String,
  medium: NotifyMedium,
  paused_at: Option<DateTime<Utc>>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
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

pub(crate) async fn find_by_id<'e, E>(
  executor: E,
  id: GroupId,
) -> Result<Option<NotifyGroup>, RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
  let row = sqlx::query_as!(
    NotifyGroupRow,
    r#"
    SELECT id, user_id, name, medium as "medium: NotifyMedium",
      paused_at, created_at, updated_at
    FROM notify_groups WHERE id = $1
    "#,
    id.as_uuid()
  )
  .fetch_optional(executor)
  .await
  .map_err(map_error)?;

  Ok(row.map(NotifyGroup::from))
}

pub(crate) async fn list_by_user_id<'e, E>(
  executor: E,
  user_id: UserId,
) -> Result<Vec<NotifyGroup>, RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
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
  .fetch_all(executor)
  .await
  .map_err(map_error)?;

  Ok(rows.into_iter().map(NotifyGroup::from).collect())
}

pub(crate) async fn list_all<'e, E>(executor: E) -> Result<Vec<NotifyGroup>, RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
  let rows = sqlx::query_as!(
    NotifyGroupRow,
    r#"
    SELECT id, user_id, name, medium as "medium: NotifyMedium", paused_at, created_at, updated_at
    FROM notify_groups
    ORDER BY created_at ASC
    "#
  )
  .fetch_all(executor)
  .await
  .map_err(map_error)?;

  Ok(rows.into_iter().map(NotifyGroup::from).collect())
}

pub(crate) async fn insert<'e, E>(executor: E, group: &NotifyGroup) -> Result<(), RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
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
  .execute(executor)
  .await
  .map_err(map_conflict_error)?;

  Ok(())
}

pub(crate) async fn update<'e, E>(executor: E, group: &NotifyGroup) -> Result<(), RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
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
  .execute(executor)
  .await
  .map_err(map_error)?;

  Ok(())
}

pub(crate) async fn delete<'e, E>(executor: E, id: GroupId) -> Result<(), RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
  sqlx::query!("DELETE FROM notify_groups WHERE id = $1", id.as_uuid())
    .execute(executor)
    .await
    .map_err(map_error)?;

  Ok(())
}
