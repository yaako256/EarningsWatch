/*
backend/crates/server/src/main.rs
serverバイナリ。
HTTPサーバの起動とDI組み立てをする
*/

// 標準ライブラリ
use std::sync::Arc;

// 外部ライブラリ
// トレイト型ロードのためにuse
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
  // 設定の読み込み
  let settings = config::load().expect("failed to load config");

  // デバッグ用にsettingsを出力
  // println!("{:#?}", settings);

  // server起動時はSqlLayerのみ登録
  let (sql_layer, _writer_handle) =
    logging::SqlLayer::new(logging::LogProcess::Server, logging::ConsoleSink);

  // レイヤの追加
  tracing_subscriber::registry()
    // ログレベルの設定(デフォルトでdebug)
    .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug".into()))
    // SQLレイヤ
    .with(sql_layer)
    // 標準出力
    .with(tracing_subscriber::fmt::layer())
    .init();

  // poolの作成
  let pool = infra::create_pool(&settings.database.url)
    .await
    .expect("failed to connect to database");

  // ----------------
  // Stateの組み立て
  // ----------------
  // PostgresSQLで組み立てる
  let user_repository: Arc<dyn repository::UserRepository> =
    Arc::new(infra::PgUserRepository::new(pool.clone()));
  let refresh_token_repository: Arc<dyn repository::RefreshTokenRepository> =
    Arc::new(infra::PgRefreshTokenRepository::new(pool.clone()));

  // AppStateにまとめる
  let state = api::state::AppState {
    user_repository,
    refresh_token_repository,
    jwt_secret: settings.jwt.secret.clone(),
    access_token_ttl_minutes: settings.jwt.access_token_ttl_minutes,
    refresh_token_ttl_days: settings.jwt.refresh_token_ttl_days,
    cookie_secure: settings.cookie.secure,
  };

  // ルータ組み立て
  let app = api::router::build_router(state);

  // アドレス組み立て
  let addr = format!("{}:{}", settings.server.host, settings.server.port);

  // 指定したアドレスでTCPリスナー(通信窓口)をバインド
  let listener = tokio::net::TcpListener::bind(&addr)
    .await
    .expect("failed to bind address");

  // デバッグ: SqlLayerの動作確認
  // for i in 1..=50 - 2 - 4 {
  //   tracing::info!(index = i, "SqlLayer test");
  // }
  // tracing::debug!("tracing debug test");
  // tracing::info!("tracing info test");
  // tracing::warn!("tracing warn test");
  // tracing::error!("tracing error test");

  // 起動確認用ログ
  tracing::info!("Starting EarningWatch server");
  tracing::info!(addr, "server starting");

  // Axumサーバーを起動してリクエストの待機を開始
  axum::serve(listener, app).await.expect("server error");
}
