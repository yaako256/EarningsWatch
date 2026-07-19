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
use super::queries::notify_queue;
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
    notify_queue::insert_monitor_marker(&self.pool).await
  }

  async fn delete_monitor_marker(&self) -> Result<(), RepositoryError> {
    notify_queue::delete_monitor_marker(&self.pool).await
  }

  async fn monitor_marker_exists(&self) -> Result<bool, RepositoryError> {
    notify_queue::monitor_marker_exists(&self.pool).await
  }

  async fn replace_data_rows(&self, entries: &[NotifyQueueEntry]) -> Result<(), RepositoryError> {
    let mut tx = self.pool.begin().await.map_err(map_error)?;

    notify_queue::replace_data_rows(&mut tx, entries).await?;

    tx.commit().await.map_err(map_error)?;

    Ok(())
  }

  async fn list_ready(&self) -> Result<Vec<NotifyQueueEntry>, RepositoryError> {
    notify_queue::list_ready(&self.pool).await
  }

  async fn update_status(&self, id: i64, status: NotifyStatus) -> Result<(), RepositoryError> {
    notify_queue::update_status(&self.pool, id, status).await
  }
}
