// crates/cli/src/commands/notify.rs
use sqlx::PgPool;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub async fn run(pool: &PgPool, webhook_enc_key: &[u8], retry_settings: &config::RetrySettings) {
  // SqlLayer+MemoryLayerを登録する
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
    .with(sql_layer)
    .with(memory_layer)
    .with(tracing_subscriber::fmt::layer())
    .init();

  let queue_repo = infra::PgNotifyQueueRepository::new(pool.clone());
  let group_repo = infra::PgNotifyGroupRepository::new(pool.clone());
  let filter_repo = infra::PgNotifyFilterRepository::new(pool.clone());
  let discord_config_repo = infra::PgNotifyDiscordConfigRepository::new(pool.clone());
  let history_repo = infra::PgNotifyHistoryRepository::new(pool.clone());
  let system_run_repo = infra::PgSystemRunRepository::new(pool.clone());

  match app::run_notify(
    &queue_repo,
    &group_repo,
    &filter_repo,
    &discord_config_repo,
    &history_repo,
    &system_run_repo,
    webhook_enc_key,
    retry_settings,
  )
  .await
  {
    Ok(result) => {
      println!(
        "notify completed: total_send_count={}, success_send_count={}, duration_ms={}",
        result.total_send_count, result.success_send_count, result.duration_ms
      );
    }
    Err(e) => {
      eprintln!("notify failed: {e}");
      std::process::exit(1);
    }
  }

  // プロセス終了前に両レイヤーをflushする
  sql_layer_for_flush.flush_now();
  memory_layer_for_flush.flush().await;
}
