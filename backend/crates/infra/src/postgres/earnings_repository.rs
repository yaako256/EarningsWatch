/*
backend/crates/infra/src/postgres/earnings_repository.rs
決算情報テーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;

// 内部ライブラリ
use earnings::{Earnings, EarningsEvaluation, EarningsRecord, EarningsSource};
use repository::{EarningsRepository, RepositoryError};

// 自クレート
use crate::error_mapping::{map_conflict_error, map_error};

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
  ) -> Result<Option<EarningsRecord>, RepositoryError> {
    let row = sqlx::query_as!(
      EarningsRow,
      r#"
      SELECT id, ticker, company_name, published_at, title, url, summary,
              evaluation as "evaluation: EarningsEvaluation",
              fingerprint,
              source as "source: EarningsSource"
      FROM earnings WHERE fingerprint = $1
      "#,
      fingerprint
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(row.map(EarningsRecord::from))
  }

  async fn list_recent_fingerprints(&self, limit: u32) -> Result<Vec<String>, RepositoryError> {
    // design/03-features/scraping.md 8章: 直近N件のfingerprintのみを既知判定に使う(件数ベース1本化)
    let rows = sqlx::query_scalar!(
      r#"SELECT fingerprint FROM earnings ORDER BY published_at DESC LIMIT $1"#,
      limit as i64
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(rows)
  }

  async fn list(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<EarningsRecord>, i64), RepositoryError> {
    let limit = per_page as i64;
    let offset = page.saturating_sub(1) as i64 * limit;

    let rows = sqlx::query_as!(
      EarningsRow,
      r#"
      SELECT id, ticker, company_name, published_at, title, url, summary,
              evaluation as "evaluation: EarningsEvaluation",
              fingerprint,
              source as "source: EarningsSource"
      FROM earnings
      ORDER BY published_at DESC
      LIMIT $1 OFFSET $2
      "#,
      limit,
      offset
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    let total_count = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM earnings"#)
      .fetch_one(&self.pool)
      .await
      .map_err(map_error)?;

    Ok((
      rows.into_iter().map(EarningsRecord::from).collect(),
      total_count,
    ))
  }

  async fn count_by_date(
    &self,
    from: chrono::DateTime<chrono::Utc>,
    to: chrono::DateTime<chrono::Utc>,
  ) -> Result<Vec<(chrono::NaiveDate, i64)>, RepositoryError> {
    // JST変換はapp層の責務とする(Phase 4引き継ぎ事項参照)。ここではUTCのDATE()で集計するのみ。
    let rows = sqlx::query!(
      r#"
      SELECT DATE(published_at) as "date!", COUNT(*) as "count!"
      FROM earnings
      WHERE published_at >= $1 AND published_at < $2
      GROUP BY DATE(published_at)
      ORDER BY DATE(published_at)
      "#,
      from,
      to
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(rows.into_iter().map(|r| (r.date, r.count)).collect())
  }

  async fn insert_many(
    &self,
    items: &[Earnings],
    fingerprints: &[String],
  ) -> Result<Vec<EarningsRecord>, RepositoryError> {
    if items.len() != fingerprints.len() {
      return Err(RepositoryError::Other(
        "itemsとfingerprintsの件数が一致しません".to_string(),
      ));
    }

    let mut tx = self.pool.begin().await.map_err(map_error)?;
    let mut records = Vec::with_capacity(items.len());

    for (item, fingerprint) in items.iter().zip(fingerprints) {
      let row = sqlx::query_as!(
        EarningsRow,
        r#"
        INSERT INTO earnings (ticker, company_name, published_at, title, url, summary, evaluation, fingerprint, source)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, ticker, company_name, published_at, title, url, summary,
                  evaluation as "evaluation: EarningsEvaluation",
                  fingerprint,
                  source as "source: EarningsSource"
        "#,
        item.ticker,
        item.company_name,
        item.published_at,
        item.title,
        item.url,
        item.summary,
        item.evaluation as EarningsEvaluation,
        fingerprint,
        EarningsSource::Kabuyoho as EarningsSource,
      )
      .fetch_one(&mut *tx)
      .await
      .map_err(map_conflict_error)?;

      records.push(EarningsRecord::from(row));
    }

    tx.commit().await.map_err(map_error)?;

    Ok(records)
  }
}
