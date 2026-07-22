/*
backend/crates/cli/src/commands/monitor.rs
決算情報を収集するサブコマンドのエントリ場所
*/

// 外部クレート
use sqlx::PgPool;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub async fn run(pool: &PgPool, recent_fingerprint_limit: u32) {
  // monitor起動時にSqlLayer+MemoryLayerを登録する
  let (sql_layer, _writer_handle) = logging::SqlLayer::new(
    logging::LogProcess::Monitor,
    logging::PgSink::new(pool.clone()),
  );
  let sql_layer_for_flush = sql_layer.clone();

  // ConsoleWarnNotifySinkは仮実装のまま(flashのDiscord実送信への差し替えは後で)
  let memory_layer =
    logging::MemoryLayer::new(logging::LogProcess::Monitor, logging::ConsoleWarnNotifySink);
  let memory_layer_for_flush = memory_layer.clone();

  tracing_subscriber::registry()
    // ログレベルの設定(デフォルトでdebug)
    .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug".into()))
    // SQLレイヤ
    .with(sql_layer)
    // メモリレイヤ
    .with(memory_layer)
    // 標準出力
    .with(tracing_subscriber::fmt::layer())
    .init();

  let scraper = scraper::DebugScraper;
  let earnings_repo = infra::PgEarningsRepository::new(pool.clone());
  let queue_repo = infra::PgNotifyQueueRepository::new(pool.clone());
  let system_run_repo = infra::PgSystemRunRepository::new(pool.clone());

  let result = app::run_monitor(
    &scraper,
    &earnings_repo,
    &queue_repo,
    &system_run_repo,
    recent_fingerprint_limit,
  )
  .await;

  if let Err(e) = &result {
    // 暫定: どのステップで失敗したかはここでは分からない
    // 後で書く場所にerror!を出させる方針に変更する
    tracing::error!(error = %e, "monitor failed");
  }

  // プロセス終了前に両レイヤーをflushする
  sql_layer_for_flush.flush_now().await;
  memory_layer_for_flush.flush().await;

  // flush後にexit codeだけ制御する
  if result.is_err() {
    std::process::exit(1);
  }
}
