/*
backend/crates/infra/src/postgres/notify_queue_repository.rs
送信キューテーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;

// 内部ライブラリ
use earnings::{EarningsEvaluation, EarningsSource};
use repository::{NotifyQueueRepository, RepositoryError};
use subscription::{NotifyQueueEntry, NotifyStatus};

// 自クレート
use crate::error_mapping::map_error;

pub struct PgNotifyQueueRepository {
  pool: PgPool,
}

impl PgNotifyQueueRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

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
    // is_monitor_marker=falseのデータ行のみをこの型に変換する前提(list_ready等はWHEREで絞り込み済み)。
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

#[async_trait]
impl NotifyQueueRepository for PgNotifyQueueRepository {
  async fn insert_monitor_marker(&self) -> Result<(), RepositoryError> {
    sqlx::query!(
      r#"
      INSERT INTO notify_queue (is_monitor_marker, fetched_at, status)
      VALUES (TRUE, now(), 'ready')
      "#
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;
    Ok(())
  }

  async fn delete_monitor_marker(&self) -> Result<(), RepositoryError> {
    sqlx::query!("DELETE FROM notify_queue WHERE is_monitor_marker = TRUE")
      .execute(&self.pool)
      .await
      .map_err(map_error)?;
    Ok(())
  }

  async fn monitor_marker_exists(&self) -> Result<bool, RepositoryError> {
    let count = sqlx::query_scalar!(
      r#"SELECT COUNT(*) as "count!" FROM notify_queue WHERE is_monitor_marker = TRUE"#
    )
    .fetch_one(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(count > 0)
  }

  async fn replace_data_rows(&self, entries: &[NotifyQueueEntry]) -> Result<(), RepositoryError> {
    let mut tx = self.pool.begin().await.map_err(map_error)?;

    sqlx::query!("DELETE FROM notify_queue WHERE is_monitor_marker = FALSE")
      .execute(&mut *tx)
      .await
      .map_err(map_error)?;

    for entry in entries {
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
      .execute(&mut *tx)
      .await
      .map_err(map_error)?;
    }

    tx.commit().await.map_err(map_error)?;
    Ok(())
  }

  async fn list_ready(&self) -> Result<Vec<NotifyQueueEntry>, RepositoryError> {
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
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    rows.into_iter().map(NotifyQueueEntry::try_from).collect()
  }

  async fn update_status(&self, id: i64, status: NotifyStatus) -> Result<(), RepositoryError> {
    sqlx::query!(
      "UPDATE notify_queue SET status = $2 WHERE id = $1",
      id,
      status as NotifyStatus
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }
}
