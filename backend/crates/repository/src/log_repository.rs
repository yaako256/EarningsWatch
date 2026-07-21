/*
backend/crates/repository/src/log_repository.rs
ログテーブルを司るリポジトリ
*/

// 外部クレート
use async_trait::async_trait;
use chrono::{DateTime, Utc};

// 内部ライブラリ
use logging::{LogEntry, LogLevel, LogProcess};

// 自クレート
use crate::RepositoryError;

pub struct ListLogsFilter {
  pub from: Option<DateTime<Utc>>,
  pub to: Option<DateTime<Utc>>,
  pub level: Option<LogLevel>,
  pub process: Option<LogProcess>,
}

#[async_trait]
pub trait LogRepository: Send + Sync {
  async fn list(
    &self,
    filter: &ListLogsFilter,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<LogEntry>, i64), RepositoryError>;
}
