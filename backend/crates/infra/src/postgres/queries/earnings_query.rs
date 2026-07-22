// backend/crates/infra/src/postgres/queries/earnings.rs

use chrono::NaiveDate;
use earnings::{Earnings, EarningsEvaluation, EarningsRecord, EarningsSource};
use repository::EarningsListFilter;
use repository::{RepositoryError, RepositoryResult};
use sqlx::{Executor, Postgres, Transaction};

use crate::error_mapping::{map_conflict_error, map_error};

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

pub(crate) async fn find_by_fingerprint<'e, E>(
  executor: E,
  fingerprint: &str,
) -> RepositoryResult<Option<EarningsRecord>>
where
  E: Executor<'e, Database = Postgres>,
{
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
  .fetch_optional(executor)
  .await
  .map_err(map_error)?;

  Ok(row.map(EarningsRecord::from))
}

pub(crate) async fn list_recent_fingerprints<'e, E>(
  executor: E,
  limit: u32,
) -> RepositoryResult<Vec<String>>
where
  E: Executor<'e, Database = Postgres>,
{
  // design/03-features/scraping.md 8章: 直近N件のfingerprintのみを既知判定に使う(件数ベース1本化)
  let rows = sqlx::query_scalar!(
    r#"SELECT fingerprint FROM earnings ORDER BY published_at DESC LIMIT $1"#,
    limit as i64
  )
  .fetch_all(executor)
  .await
  .map_err(map_error)?;

  Ok(rows)
}

pub(crate) async fn list<'e, E>(
  executor: E,
  page: u32,
  per_page: u32,
) -> RepositoryResult<(Vec<EarningsRecord>, i64)>
where
  E: Executor<'e, Database = Postgres>,
{
  let limit = per_page as i64;
  let offset = page.saturating_sub(1) as i64 * limit;

  // executorの所有権問題を解決するため、
  // クエリを1つにまとめた
  let rows = sqlx::query!(
    r#"
    SELECT
      id,
      ticker,
      company_name,
      published_at,
      title,
      url,
      summary,
      evaluation as "evaluation: EarningsEvaluation",
      fingerprint,
      source as "source: EarningsSource",
      COUNT(*) OVER() as "total_count!"
    FROM earnings
    ORDER BY published_at DESC
    LIMIT $1 OFFSET $2
    "#,
    limit,
    offset
  )
  .fetch_all(executor)
  .await
  .map_err(map_error)?;

  let total_count = rows.first().map(|row| row.total_count).unwrap_or(0);

  let records = rows
    .into_iter()
    .map(|row| {
      EarningsRecord::from(EarningsRow {
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
      })
    })
    .collect();

  Ok((records, total_count))
}

pub(crate) async fn count_by_date<'e, E>(
  executor: E,
  from: chrono::DateTime<chrono::Utc>,
  to: chrono::DateTime<chrono::Utc>,
) -> RepositoryResult<Vec<(NaiveDate, i64)>>
where
  E: Executor<'e, Database = Postgres>,
{
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
  .fetch_all(executor)
  .await
  .map_err(map_error)?;

  Ok(rows.into_iter().map(|r| (r.date, r.count)).collect())
}

pub(crate) async fn insert_one<'e, E>(
  executor: E,
  item: &Earnings,
  fingerprint: &str,
) -> RepositoryResult<EarningsRecord>
where
  E: Executor<'e, Database = Postgres>,
{
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
  .fetch_one(executor)
  .await
  .map_err(map_conflict_error)?;

  Ok(EarningsRecord::from(row))
}

pub(crate) async fn insert_many(
  tx: &mut Transaction<'_, Postgres>,
  items: &[Earnings],
  fingerprints: &[String],
) -> RepositoryResult<Vec<EarningsRecord>> {
  if items.len() != fingerprints.len() {
    return Err(RepositoryError::Other(
      "itemsとfingerprintsの件数が一致しません".to_string(),
    ));
  }

  let mut records = Vec::with_capacity(items.len());

  for (item, fingerprint) in items.iter().zip(fingerprints) {
    records.push(insert_one(&mut **tx, item, fingerprint).await?);
  }

  Ok(records)
}

pub(crate) async fn list_filtered<'e, E>(
  executor: E,
  filter: &EarningsListFilter,
  page: u32,
  per_page: u32,
) -> RepositoryResult<(Vec<EarningsRecord>, i64)>
where
  E: Executor<'e, Database = Postgres>,
{
  let limit = per_page as i64;
  let offset = page.saturating_sub(1) as i64 * limit;

  // 所有権の問題より、list同様、クエリを1つにまとめた
  let rows = sqlx::query!(
    r#"
      SELECT
        id,
        ticker,
        company_name,
        published_at,
        title,
        url,
        summary,
        evaluation as "evaluation: EarningsEvaluation",
        fingerprint,
        source as "source: EarningsSource",
        COUNT(*) OVER() as "total_count!"
      FROM earnings
      WHERE ($1::text IS NULL OR ticker = $1)
        AND ($2::text IS NULL OR company_name ILIKE '%' || $2 || '%')
        AND ($3::earnings_evaluation IS NULL OR evaluation = $3)
        AND ($4::timestamptz IS NULL OR published_at >= $4)
        AND ($5::timestamptz IS NULL OR published_at < $5)
      ORDER BY published_at DESC
      LIMIT $6 OFFSET $7
      "#,
    filter.ticker,
    filter.company_name,
    filter.evaluation as Option<EarningsEvaluation>,
    filter.from,
    filter.to,
    limit,
    offset
  )
  .fetch_all(executor)
  .await
  .map_err(map_error)?;

  let total_count = rows.first().map(|row| row.total_count).unwrap_or(0);

  let records = rows
    .into_iter()
    .map(|row| {
      EarningsRecord::from(EarningsRow {
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
      })
    })
    .collect();

  Ok((records, total_count))
}

pub(crate) async fn count_all<'e, E>(executor: E) -> RepositoryResult<i64>
where
  E: Executor<'e, Database = Postgres>,
{
  sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM earnings"#)
    .fetch_one(executor)
    .await
    .map_err(map_error)
}

pub(crate) async fn summary_daily_counts_jst<'e, E>(
  executor: E,
  from: Option<chrono::DateTime<chrono::Utc>>,
  to: Option<chrono::DateTime<chrono::Utc>>,
) -> RepositoryResult<Vec<(chrono::NaiveDate, i64)>>
where
  E: Executor<'e, Database = Postgres>,
{
  let rows = sqlx::query!(
    r#"
      SELECT (published_at AT TIME ZONE 'Asia/Tokyo')::date as "date_jst!", COUNT(*) as "count!"
      FROM earnings
      WHERE ($1::timestamptz IS NULL OR published_at >= $1)
        AND ($2::timestamptz IS NULL OR published_at < $2)
      GROUP BY (published_at AT TIME ZONE 'Asia/Tokyo')::date
      ORDER BY (published_at AT TIME ZONE 'Asia/Tokyo')::date
      "#,
    from,
    to
  )
  .fetch_all(executor)
  .await
  .map_err(map_error)?;

  Ok(rows.into_iter().map(|r| (r.date_jst, r.count)).collect())
}
