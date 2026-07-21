/*
backend/crates/infra/src/postgres/earnings_repository.rs
決算情報テーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;

// 内部ライブラリ
use earnings::{Earnings, EarningsEvaluation, EarningsRecord, EarningsSource};
use repository::{EarningsRepository, RepositoryError, RepositoryResult};

// 自クレート
use super::queries::earnings_query;
use crate::error_mapping::map_error;

pub struct PgEarningsRepository {
  pool: PgPool,
}

impl PgEarningsRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

struct EarningsRow {
  id: i64,
  ticker: String,
  company_name: String,
  published_at: chrono::DateTime<chrono::Utc>,
  title: String,
  url: String,
  summary: String,
  evaluation: EarningsEvaluation,
  fingerprint: String,
  source: EarningsSource,
}

impl From<EarningsRow> for EarningsRecord {
  fn from(row: EarningsRow) -> Self {
    EarningsRecord {
      id: row.id,
      ticker: row.ticker,
      company_name: row.company_name,
      published_at: row.published_at,
      title: row.title,
      url: row.url,
      summary: row.summary,
      evaluation: row.evaluation,
      fingerprint: row.fingerprint,
      source: row.source,
    }
  }
}

#[async_trait]
impl EarningsRepository for PgEarningsRepository {
  async fn find_by_fingerprint(
    &self,
    fingerprint: &str,
  ) -> RepositoryResult<Option<EarningsRecord>> {
    earnings_query::find_by_fingerprint(&self.pool, fingerprint).await
  }

  async fn list_recent_fingerprints(&self, limit: u32) -> RepositoryResult<Vec<String>> {
    earnings_query::list_recent_fingerprints(&self.pool, limit).await
  }

  async fn list(&self, page: u32, per_page: u32) -> RepositoryResult<(Vec<EarningsRecord>, i64)> {
    earnings_query::list(&self.pool, page, per_page).await
  }

  async fn count_by_date(
    &self,
    from: chrono::DateTime<chrono::Utc>,
    to: chrono::DateTime<chrono::Utc>,
  ) -> RepositoryResult<Vec<(chrono::NaiveDate, i64)>> {
    earnings_query::count_by_date(&self.pool, from, to).await
  }

  async fn insert_many(
    &self,
    items: &[Earnings],
    fingerprints: &[String],
  ) -> RepositoryResult<Vec<EarningsRecord>> {
    if items.len() != fingerprints.len() {
      return Err(RepositoryError::Other(
        "itemsとfingerprintsの件数が一致しません".to_string(),
      ));
    }

    let mut tx = self.pool.begin().await.map_err(map_error)?;

    let records = earnings_query::insert_many(&mut tx, items, fingerprints).await?;

    tx.commit().await.map_err(map_error)?;

    Ok(records)
  }
}
