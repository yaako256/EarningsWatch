// backend/crates/infra/src/postgres/queries/notify_queue.rs

use earnings::{EarningsEvaluation, EarningsSource};
use repository::RepositoryError;
use sqlx::{Executor, Postgres, Transaction};
use subscription::{NotifyQueueEntry, NotifyStatus};

use crate::error_mapping::{map_conflict_error, map_error};

struct NotifyQueueRow {
  id: i64,
  fingerprint: Option<String>,
  source: Option<EarningsSource>,
  fetched_at: chrono::DateTime<chrono::Utc>,
  ticker: Option<String>,
  company_name: Option<String>,
  published_at: Option<chrono::DateTime<chrono::Utc>>,
  title: Option<String>,
  url: Option<String>,
  summary: Option<String>,
  evaluation: Option<EarningsEvaluation>,
  status: NotifyStatus,
}

impl TryFrom<NotifyQueueRow> for NotifyQueueEntry {
  type Error = RepositoryError;

  fn try_from(row: NotifyQueueRow) -> Result<Self, Self::Error> {
    Ok(NotifyQueueEntry {
      id: row.id,
      fingerprint: row
        .fingerprint
        .ok_or_else(|| RepositoryError::Other("fingerprintがNULLの行です".into()))?,
      source: row
        .source
        .ok_or_else(|| RepositoryError::Other("sourceがNULLの行です".into()))?,
      fetched_at: row.fetched_at,
      ticker: row
        .ticker
        .ok_or_else(|| RepositoryError::Other("tickerがNULLの行です".into()))?,
      company_name: row
        .company_name
        .ok_or_else(|| RepositoryError::Other("company_nameがNULLの行です".into()))?,
      published_at: row
        .published_at
        .ok_or_else(|| RepositoryError::Other("published_atがNULLの行です".into()))?,
      title: row
        .title
        .ok_or_else(|| RepositoryError::Other("titleがNULLの行です".into()))?,
      url: row
        .url
        .ok_or_else(|| RepositoryError::Other("urlがNULLの行です".into()))?,
      summary: row
        .summary
        .ok_or_else(|| RepositoryError::Other("summaryがNULLの行です".into()))?,
      evaluation: row
        .evaluation
        .ok_or_else(|| RepositoryError::Other("evaluationがNULLの行です".into()))?,
      status: row.status,
    })
  }
}

pub(crate) async fn insert_monitor_marker<'e, E>(executor: E) -> Result<(), RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
  sqlx::query!(
    r#"
    INSERT INTO notify_queue (is_monitor_marker, fetched_at, status)
    VALUES (TRUE, now(), 'ready')
    "#
  )
  .execute(executor)
  .await
  .map_err(map_error)?;
  Ok(())
}

pub(crate) async fn delete_monitor_marker<'e, E>(executor: E) -> Result<(), RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
  sqlx::query!("DELETE FROM notify_queue WHERE is_monitor_marker = TRUE")
    .execute(executor)
    .await
    .map_err(map_error)?;
  Ok(())
}

pub(crate) async fn monitor_marker_exists<'e, E>(executor: E) -> Result<bool, RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
  let count = sqlx::query_scalar!(
    r#"SELECT COUNT(*) as "count!" FROM notify_queue WHERE is_monitor_marker = TRUE"#
  )
  .fetch_one(executor)
  .await
  .map_err(map_error)?;

  Ok(count > 0)
}

pub(crate) async fn list_ready<'e, E>(executor: E) -> Result<Vec<NotifyQueueEntry>, RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
  let rows = sqlx::query_as!(
    NotifyQueueRow,
    r#"
    SELECT id, fingerprint, source as "source: EarningsSource", fetched_at,
      ticker, company_name, published_at, title, url, summary,
      evaluation as "evaluation: EarningsEvaluation",
      status as "status: NotifyStatus"
    FROM notify_queue
    WHERE is_monitor_marker = FALSE AND status = 'ready'
    ORDER BY published_at ASC
    "#
  )
  .fetch_all(executor)
  .await
  .map_err(map_error)?;

  rows.into_iter().map(NotifyQueueEntry::try_from).collect()
}

pub(crate) async fn update_status<'e, E>(
  executor: E,
  id: i64,
  status: NotifyStatus,
) -> Result<(), RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
  sqlx::query!(
    "UPDATE notify_queue SET status = $2 WHERE id = $1",
    id,
    status as NotifyStatus
  )
  .execute(executor)
  .await
  .map_err(map_error)?;

  Ok(())
}

pub(crate) async fn delete_data_rows<'e, E>(executor: E) -> Result<(), RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
  sqlx::query!("DELETE FROM notify_queue WHERE is_monitor_marker = FALSE")
    .execute(executor)
    .await
    .map_err(map_error)?;

  Ok(())
}

pub(crate) async fn insert_data_row<'e, E>(
  executor: E,
  entry: &NotifyQueueEntry,
) -> Result<(), RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
  sqlx::query!(
    r#"
    INSERT INTO notify_queue
      (fingerprint, is_monitor_marker, source, fetched_at, ticker, company_name,
        published_at, title, url, summary, evaluation, status)
    VALUES ($1, FALSE, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    "#,
    entry.fingerprint,
    entry.source as EarningsSource,
    entry.fetched_at,
    entry.ticker,
    entry.company_name,
    entry.published_at,
    entry.title,
    entry.url,
    entry.summary,
    entry.evaluation as EarningsEvaluation,
    entry.status as NotifyStatus,
  )
  .execute(executor)
  .await
  .map_err(map_conflict_error)?;

  Ok(())
}

pub(crate) async fn replace_data_rows(
  tx: &mut Transaction<'_, Postgres>,
  entries: &[NotifyQueueEntry],
) -> Result<(), RepositoryError> {
  delete_data_rows(&mut **tx).await?;

  for entry in entries {
    insert_data_row(&mut **tx, entry).await?;
  }

  Ok(())
}
