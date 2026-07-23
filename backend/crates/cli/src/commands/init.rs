/*
backend/crates/cli/src/commands/init.rs
初回起動時の決算情報取得用サブコマンド
*/

// 外部クレート
use sqlx::PgPool;

pub async fn run(pool: &PgPool, list_page_interval_seconds: u64, detail_interval_seconds: u64) {
  // スクレイパーフォルダのパスを取得
  // (環境ごとに分かれるためcomposeで環境変数定義)
  let scraper_dir_path =
    std::env::var("EARNINGSWATCH_SCRAPER_DIR").expect("failed to load scraper_dir_path");

  // init処理は特に負荷が高そうであり
  // 制限時間も特にないため、通常の2倍の時間をかける
  let scraper =
    scraper::KabuyohoScraper::new(list_page_interval_seconds * 2, detail_interval_seconds * 2);

  let earnings_repo = infra::PgEarningsRepository::new(pool.clone());

  match app::run_init(&scraper, &earnings_repo, &scraper_dir_path).await {
    Ok(result) => println!(
      "init completed: initialized_count={}",
      result.initialized_count
    ),
    Err(e) => {
      eprintln!("init failed: {e}");
      std::process::exit(1);
    }
  }
}
