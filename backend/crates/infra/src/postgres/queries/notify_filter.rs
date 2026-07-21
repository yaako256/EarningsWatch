// backend/crates/infra/src/postgres/queries/notify_filter.rs

use chrono::{DateTime, Utc};
use identity::{FilterId, GroupId};
use repository::RepositoryResult;
use sqlx::{Executor, Postgres, Transaction};
use subscription::NotifyFilter;
use uuid::Uuid;

use crate::error_mapping::{map_conflict_error, map_error};

struct NotifyFilterRow {
  id: Uuid,
  group_id: Uuid,
  ticker: String,
  company_name: String,
  notes: Option<String>,
  enabled: bool,
  created_at: DateTime<Utc>,
}

impl From<NotifyFilterRow> for NotifyFilter {
  fn from(row: NotifyFilterRow) -> Self {
    NotifyFilter {
      id: FilterId::from_uuid(row.id),
      group_id: GroupId::from_uuid(row.group_id),
      ticker: row.ticker,
      company_name: row.company_name,
      notes: row.notes,
      enabled: row.enabled,
      created_at: row.created_at,
    }
  }
}

pub(crate) async fn find_by_id<'e, E>(
  executor: E,
  id: FilterId,
) -> RepositoryResult<Option<NotifyFilter>>
where
  E: Executor<'e, Database = Postgres>,
{
  let row = sqlx::query_as!(
    NotifyFilterRow,
    r#"
    SELECT id, group_id, ticker, company_name, notes, enabled, created_at
    FROM notify_filters WHERE id = $1
    "#,
    id.as_uuid()
  )
  .fetch_optional(executor)
  .await
  .map_err(map_error)?;

  Ok(row.map(NotifyFilter::from))
}

pub(crate) async fn list_by_group_id<'e, E>(
  executor: E,
  group_id: GroupId,
) -> RepositoryResult<Vec<NotifyFilter>>
where
  E: Executor<'e, Database = Postgres>,
{
  let rows = sqlx::query_as!(
    NotifyFilterRow,
    r#"
    SELECT id, group_id, ticker, company_name, notes, enabled, created_at
    FROM notify_filters WHERE group_id = $1 ORDER BY created_at ASC
    "#,
    group_id.as_uuid()
  )
  .fetch_all(executor)
  .await
  .map_err(map_error)?;

  Ok(rows.into_iter().map(NotifyFilter::from).collect())
}

pub(crate) async fn insert<'e, E>(executor: E, filter: &NotifyFilter) -> RepositoryResult<()>
where
  E: Executor<'e, Database = Postgres>,
{
  sqlx::query!(
    r#"
    INSERT INTO notify_filters (id, group_id, ticker, company_name, notes, enabled, created_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    "#,
    filter.id.as_uuid(),
    filter.group_id.as_uuid(),
    filter.ticker,
    filter.company_name,
    filter.notes,
    filter.enabled,
    filter.created_at
  )
  .execute(executor)
  .await
  .map_err(map_conflict_error)?;

  Ok(())
}

pub(crate) async fn update<'e, E>(executor: E, filter: &NotifyFilter) -> RepositoryResult<()>
where
  E: Executor<'e, Database = Postgres>,
{
  sqlx::query!(
    r#"
    UPDATE notify_filters
    SET ticker = $2, company_name = $3, notes = $4, enabled = $5
    WHERE id = $1
    "#,
    filter.id.as_uuid(),
    filter.ticker,
    filter.company_name,
    filter.notes,
    filter.enabled
  )
  .execute(executor)
  .await
  .map_err(map_error)?;

  Ok(())
}

pub(crate) async fn delete<'e, E>(executor: E, id: FilterId) -> RepositoryResult<()>
where
  E: Executor<'e, Database = Postgres>,
{
  sqlx::query!("DELETE FROM notify_filters WHERE id = $1", id.as_uuid())
    .execute(executor)
    .await
    .map_err(map_error)?;

  Ok(())
}

pub(crate) async fn replace_all_for_group(
  tx: &mut Transaction<'_, Postgres>,
  group_id: GroupId,
  filters: &[NotifyFilter],
) -> RepositoryResult<()> {
  delete_by_group_id(&mut **tx, group_id).await?;

  for filter in filters {
    insert(&mut **tx, filter).await?;
  }

  Ok(())
}

pub(crate) async fn delete_by_group_id<'e, E>(
  executor: E,
  group_id: GroupId,
) -> RepositoryResult<()>
where
  E: Executor<'e, Database = Postgres>,
{
  sqlx::query!(
    "DELETE FROM notify_filters WHERE group_id = $1",
    group_id.as_uuid()
  )
  .execute(executor)
  .await
  .map_err(map_error)?;

  Ok(())
}
