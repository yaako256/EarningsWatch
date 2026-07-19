/*
backend/crates/infra/src/pool.rs
共通ヘルパー
*/

// 外部クレート
use sqlx::postgres::{PgPool, PgPoolOptions};

/// server/cli共通のPgPool生成ヘルパー
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
  PgPoolOptions::new()
    .max_connections(10)
    .connect(database_url)
    .await
}
