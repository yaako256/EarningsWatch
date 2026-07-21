/*
backend/crates/infra/src/postgres/log_repository.rs
ログテーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;

// 内部ライブラリ
use logging::{LogEntry, LogLevel, LogProcess};
use repository::{ListLogsFilter, LogRepository, RepositoryError};

// 自クレート
use crate::error_mapping::map_error;

pub struct PgLogRepository {
  pool: PgPool,
}

impl PgLogRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

struct LogRow {
  id: i64,
  timestamp: chrono::DateTime<chrono::Utc>,
  level: LogLevel,
  process: LogProcess,
  target: String,
  message: Option<String>,
  fields: serde_json::Value,
}

impl From<LogRow> for LogEntry {
  fn from(row: LogRow) -> Self {
    LogEntry {
      id: row.id,
      timestamp: row.timestamp,
      level: row.level,
      process: row.process,
      target: row.target,
      message: row.message,
      fields: row.fields,
    }
  }
}

#[async_trait]
impl LogRepository for PgLogRepository {
  async fn list(
    &self,
    filter: &ListLogsFilter,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<LogEntry>, i64), RepositoryError> {
    let limit = per_page as i64;
    let offset = page.saturating_sub(1) as i64 * limit;

    let rows = sqlx::query_as!(
      LogRow,
      r#"
      SELECT id, timestamp, level as "level: LogLevel", process as "process: LogProcess",
              target, message, fields
      FROM logs
      WHERE ($1::timestamptz IS NULL OR timestamp >= $1)
        AND ($2::timestamptz IS NULL OR timestamp < $2)
        AND ($3::text IS NULL OR level = $3)
        AND ($4::log_process IS NULL OR process = $4)
      ORDER BY timestamp DESC
      LIMIT $5 OFFSET $6
      "#,
      filter.from,
      filter.to,
      filter.level as Option<LogLevel>,
      filter.process as Option<LogProcess>,
      limit,
      offset
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    let total_count = sqlx::query_scalar!(
      r#"
      SELECT COUNT(*) as "count!" FROM logs
      WHERE ($1::timestamptz IS NULL OR timestamp >= $1)
        AND ($2::timestamptz IS NULL OR timestamp < $2)
        AND ($3::text IS NULL OR level = $3)
        AND ($4::log_process IS NULL OR process = $4)
      "#,
      filter.from,
      filter.to,
      filter.level as Option<LogLevel>,
      filter.process as Option<LogProcess>,
    )
    .fetch_one(&self.pool)
    .await
    .map_err(map_error)?;

    Ok((rows.into_iter().map(LogEntry::from).collect(), total_count))
  }
}
