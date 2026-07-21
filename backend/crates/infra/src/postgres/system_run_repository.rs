/*
backend/crates/infra/src/postgres/system_run_repository.rs
実行記録テーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;

// 内部ライブラリ
use repository::{RepositoryResult, SystemRunRepository};

// 自クレート
use crate::error_mapping::map_error;

pub struct PgSystemRunRepository {
  pool: PgPool,
}

impl PgSystemRunRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl SystemRunRepository for PgSystemRunRepository {
  async fn record_monitor_run(
    &self,
    run_at: chrono::DateTime<chrono::Utc>,
    duration_ms: i32,
    new_earnings_count: i32,
  ) -> RepositoryResult<()> {
    sqlx::query!(
      r#"
      INSERT INTO system_runs (run_type, run_at, duration_ms, new_earnings_count)
      VALUES ('monitor', $1, $2, $3)
      "#,
      run_at,
      duration_ms,
      new_earnings_count
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }

  async fn record_notify_run(
    &self,
    run_at: chrono::DateTime<chrono::Utc>,
    duration_ms: i32,
    total_send_count: i32,
    success_send_count: i32,
  ) -> RepositoryResult<()> {
    sqlx::query!(
      r#"
      INSERT INTO system_runs (run_type, run_at, duration_ms, total_send_count, success_send_count)
      VALUES ('notify', $1, $2, $3, $4)"#,
      run_at,
      duration_ms,
      total_send_count,
      success_send_count
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }

  async fn recent_notify_success_rate(&self, recent_n: u32) -> RepositoryResult<Option<f64>> {
    let row = sqlx::query!(
      r#"
      SELECT SUM(success_send_count) as total_success, SUM(total_send_count) as total_sent
      FROM (
        SELECT success_send_count, total_send_count
        FROM system_runs
        WHERE run_type = 'notify'
        ORDER BY run_at DESC
        LIMIT $1
      ) recent
      "#,
      recent_n as i64
    )
    .fetch_one(&self.pool)
    .await
    .map_err(map_error)?;

    match (row.total_success, row.total_sent) {
      (Some(success), Some(total)) if total > 0 => Ok(Some(success as f64 / total as f64)),
      _ => Ok(None),
    }
  }

  async fn last_monitor_run_at(&self) -> RepositoryResult<Option<chrono::DateTime<chrono::Utc>>> {
    sqlx::query_scalar!(
      r#"SELECT run_at FROM system_runs WHERE run_type = 'monitor' ORDER BY run_at DESC LIMIT 1"#
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(map_error)
  }

  async fn recent_run_durations(
    &self,
    recent_n: u32,
  ) -> RepositoryResult<Vec<(String, chrono::DateTime<chrono::Utc>, i32)>> {
    let rows = sqlx::query!(
      r#"
      SELECT run_type::text as "run_type!", run_at, duration_ms
      FROM system_runs
      ORDER BY run_at DESC
      LIMIT $1
      "#,
      recent_n as i64
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(
      rows
        .into_iter()
        .map(|r| (r.run_type, r.run_at, r.duration_ms))
        .collect(),
    )
  }
}
