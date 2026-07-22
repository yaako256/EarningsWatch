// crates/app/src/admin/admin_dashboard.rs
use chrono::{DateTime, Utc};
use repository::{EarningsRepository, SystemRunRepository};

use crate::AppError;

pub struct AdminDashboardData {
  pub total_earnings_count: i64,
  pub notify_success_rate: Option<f64>,
  pub last_monitor_run_at: Option<DateTime<Utc>>,
  pub run_durations: Vec<(String, DateTime<Utc>, i32)>, // (run_type生文字列, run_at, duration_ms)
}

pub async fn admin_dashboard(
  earnings_repo: &dyn EarningsRepository,
  system_run_repo: &dyn SystemRunRepository,
  recent_n: u32,
) -> Result<AdminDashboardData, AppError> {
  let total_earnings_count = earnings_repo.count_all().await?;
  let notify_success_rate = system_run_repo.recent_notify_success_rate(recent_n).await?;
  let last_monitor_run_at = system_run_repo.last_monitor_run_at().await?;
  let run_durations = system_run_repo.recent_run_durations(recent_n).await?;

  Ok(AdminDashboardData {
    total_earnings_count,
    notify_success_rate,
    last_monitor_run_at,
    run_durations,
  })
}
