/*
backend/crates/app/src/monitor/run_monitor.rs
monitor処理をするユースケース
*/

// 標準ライブラリ
use std::collections::HashSet;
use std::time::Instant;

// 外部クレート
use chrono::Utc;

// 内部ライブラリ
use repository::{EarningsRepository, NotifyQueueRepository, SystemRunRepository};
use scraper::ScraperService;
use subscription::{NotifyQueueEntry, NotifyStatus};

// 自クレート
use crate::AppError;

pub struct MonitorRunResult {
  pub new_earnings_count: u32,
  pub duration_ms: i32,
}

pub async fn run_monitor(
  scraper: &dyn ScraperService,
  earnings_repo: &dyn EarningsRepository,
  queue_repo: &dyn NotifyQueueRepository,
  system_run_repo: &dyn SystemRunRepository,
  recent_fingerprint_limit: u32,
) -> Result<MonitorRunResult, AppError> {
  let run_at = Utc::now();
  let start = Instant::now();

  // 1. マーカー行挿入
  queue_repo.insert_monitor_marker().await?;

  let known_fingerprints: HashSet<String> = earnings_repo
    .list_recent_fingerprints(recent_fingerprint_limit)
    .await?
    .into_iter()
    .collect();

  // スクレイピング処理(監視処理)
  let (new_earnings, new_fingerprints) = scraper
    .fetch_earning_info(known_fingerprints)
    .await
    .map_err(|e| AppError::ScraperError(e.to_string()))?;

  // テーブルに追加
  let records = earnings_repo
    .insert_many(&new_earnings, &new_fingerprints)
    .await?;

  // notify_queueへ反映
  let queue_entries: Vec<NotifyQueueEntry> = records
    .iter()
    .map(|r| NotifyQueueEntry {
      id: 0, // DB側で自動採番(9章参照)
      fingerprint: r.fingerprint.clone(),
      source: r.source,
      fetched_at: run_at,
      ticker: r.ticker.clone(),
      company_name: r.company_name.clone(),
      published_at: r.published_at,
      title: r.title.clone(),
      url: r.url.clone(),
      summary: r.summary.clone(),
      evaluation: r.evaluation,
      status: NotifyStatus::Ready,
    })
    .collect();

  queue_repo.replace_data_rows(&queue_entries).await?;

  // マーカー行削除
  queue_repo.delete_monitor_marker().await?;

  let duration_ms = start.elapsed().as_millis() as i32;
  system_run_repo
    .record_monitor_run(run_at, duration_ms, records.len() as i32)
    .await?;

  tracing::info!(
    new_earnings_count = records.len(),
    duration_ms,
    "monitor completed"
  );

  Ok(MonitorRunResult {
    new_earnings_count: records.len() as u32,
    duration_ms,
  })
}
