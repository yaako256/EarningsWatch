/*
backend/crates/server/src/main.rs
serverバイナリ。
HTTPサーバの起動とDI組み立てをする
*/

// 標準ライブラリ
use std::sync::Arc;

// 外部ライブラリ
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
// トレイト型ロードのためにpreludeでuse
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
  // 設定の読み込み
  let settings = config::load().expect("failed to load config");

  // poolの作成
  let pool = infra::create_pool(&settings.database.url)
    .await
    .expect("failed to connect to database");

  // server起動時はSqlLayerのみ登録
  let (sql_layer, _writer_handle) = logging::SqlLayer::new(
    logging::LogProcess::Server,
    logging::PgSink::new(pool.clone()),
  );

  // ロギング設定: レイヤの追加
  tracing_subscriber::registry()
    // ログレベルの設定(デフォルトでdebug)
    .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug".into()))
    // SQLレイヤ
    .with(sql_layer)
    // 標準出力
    .with(tracing_subscriber::fmt::layer())
    .init();

  // webhook_enc_keyはAppState構築時に1回だけbase64デコード
  let webhook_enc_key = STANDARD
    .decode(&settings.security.webhook_enc_key)
    .expect("EARNINGSWATCH__SECURITY__WEBHOOK_ENC_KEYのbase64デコードに失敗しました");

  // ----------------
  // Stateの組み立て
  // ----------------
  // PostgresSQLで組み立てる
  let user_repository: Arc<dyn repository::UserRepository> =
    Arc::new(infra::PgUserRepository::new(pool.clone()));
  let refresh_token_repository: Arc<dyn repository::RefreshTokenRepository> =
    Arc::new(infra::PgRefreshTokenRepository::new(pool.clone()));
  let notify_group_repository: Arc<dyn repository::NotifyGroupRepository> =
    Arc::new(infra::PgNotifyGroupRepository::new(pool.clone()));
  let notify_discord_config_repository: Arc<dyn repository::NotifyDiscordConfigRepository> =
    Arc::new(infra::PgNotifyDiscordConfigRepository::new(pool.clone()));
  let notify_slack_config_repository: Arc<dyn repository::NotifySlackConfigRepository> =
    Arc::new(infra::PgNotifySlackConfigRepository::new(pool.clone()));
  let notify_filter_repository: Arc<dyn repository::NotifyFilterRepository> =
    Arc::new(infra::PgNotifyFilterRepository::new(pool.clone()));
  let unit_of_work: Arc<dyn repository::UnitOfWork> =
    Arc::new(infra::PgUnitOfWork::new(pool.clone()));
  let earnings_repository: Arc<dyn repository::EarningsRepository> =
    Arc::new(infra::PgEarningsRepository::new(pool.clone()));
  let notify_history_repository: Arc<dyn repository::NotifyHistoryRepository> =
    Arc::new(infra::PgNotifyHistoryRepository::new(pool.clone()));
  let notify_queue_repository: Arc<dyn repository::NotifyQueueRepository> =
    Arc::new(infra::PgNotifyQueueRepository::new(pool.clone()));
  let log_repository: Arc<dyn repository::LogRepository> =
    Arc::new(infra::PgLogRepository::new(pool.clone()));
  let system_notify_config_repository: Arc<dyn repository::SystemNotifyConfigRepository> =
    Arc::new(infra::PgSystemNotifyConfigRepository::new(pool.clone()));
  let system_run_repository: Arc<dyn repository::SystemRunRepository> =
    Arc::new(infra::PgSystemRunRepository::new(pool.clone()));
  let page_repository: Arc<dyn repository::PageRepository> =
    Arc::new(infra::PgPageRepository::new(pool.clone()));

  // AppStateにまとめる
  let state = api::state::AppState {
    // リポジトリ系
    user_repository,
    refresh_token_repository,
    notify_group_repository,
    notify_discord_config_repository,
    notify_slack_config_repository,
    notify_filter_repository,
    notify_history_repository,
    notify_queue_repository,
    earnings_repository,
    log_repository,
    system_notify_config_repository,
    system_run_repository,
    page_repository,
    unit_of_work,

    // 設定系
    jwt_secret: settings.jwt.secret.clone(),
    access_token_ttl_minutes: settings.jwt.access_token_ttl_minutes,
    refresh_token_ttl_days: settings.jwt.refresh_token_ttl_days,
    cookie_secure: settings.cookie.secure,
    webhook_enc_key,
    import_settings: settings.import.clone(),
    dashboard_settings: settings.dashboard.clone(),
  };

  // ルータ組み立て
  let app = api::router::build_router(state);

  // アドレス組み立て
  let addr = format!("{}:{}", settings.server.host, settings.server.port);

  // 指定したアドレスでTCPリスナー(通信窓口)をバインド
  let listener = tokio::net::TcpListener::bind(&addr)
    .await
    .expect("failed to bind address");

  // デバッグ: SqlLayerログの動作確認
  for i in 1..=50 - 2 - 4 + 1 {
    tracing::info!(index = i, "SqlLayer test");
  }
  tracing::debug!("tracing debug test");
  tracing::info!("tracing info test");
  tracing::warn!("tracing warn test");
  tracing::error!("tracing error test");

  // 起動確認用ログ
  tracing::info!("Starting EarningWatch server");
  tracing::info!(addr, "server starting");

  // Axumサーバーを起動してリクエストの待機を開始
  axum::serve(listener, app).await.expect("server error");
}
