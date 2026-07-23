/*
backend/crates/app/src/monitor/run_init.rs
初回実行時の決算情報保存ユースケース
*/

// 標準ライブラリ
use std::collections::HashSet;
use std::path::Path;

// 内部ライブラリ
use repository::EarningsRepository;
use scraper::ScraperService;

// 外部クレート
use crate::AppError;

pub struct InitRunResult {
  pub initialized_count: u32,
}

/// 初回実行時、直近分を「既存ニュース」として記録するのみでnotify_queueには反映しない。
pub async fn run_init(
  scraper: &dyn ScraperService,
  earnings_repo: &dyn EarningsRepository,
  scraper_dir_path: impl AsRef<Path>,
) -> Result<InitRunResult, AppError> {
  let (earnings, fingerprints) = scraper
    // 既知集合を空にすることで、直近分を全件取得させる
    .fetch_earning_info(HashSet::new(), scraper_dir_path.as_ref())
    .await
    .map_err(|e| AppError::ScraperError(e.to_string()))?;

  let records = earnings_repo.insert_many(&earnings, &fingerprints).await?;

  Ok(InitRunResult {
    initialized_count: records.len() as u32,
  })
}
