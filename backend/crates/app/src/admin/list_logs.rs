/*
backend/crates/app/src/admin/list_logs.rs
ログ一覧のユースケース
*/

// 内部ライブラリ
use logging::{LogEntry, LogLevel, LogProcess};
use repository::{ListLogsFilter, LogRepository};

// 自クレート
use crate::AppError;

#[allow(clippy::too_many_arguments)]
pub async fn list_logs(
  log_repo: &dyn LogRepository,
  from: Option<chrono::DateTime<chrono::Utc>>,
  to: Option<chrono::DateTime<chrono::Utc>>,
  level: Option<LogLevel>,
  process: Option<LogProcess>,
  page: u32,
  per_page: u32,
) -> Result<(Vec<LogEntry>, i64), AppError> {
  let filter = ListLogsFilter {
    from,
    to,
    level,
    process,
  };
  Ok(log_repo.list(&filter, page, per_page).await?)
}
