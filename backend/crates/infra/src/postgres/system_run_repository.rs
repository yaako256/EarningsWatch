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
}
