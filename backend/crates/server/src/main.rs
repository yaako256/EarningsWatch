/*
backend/crates/server/src/main.rs
serverバイナリ。
HTTPサーバの起動とDI組み立てをする
*/

// 外部ライブラリ
// トレイト型ロードのためにuse
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
  // 設定の読み込み
  let settings = config::load().expect("failed to load config");

  // デバッグ用にsettingsを出力
  println!("{:#?}", settings);

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

  // ステートの作成
  let state = api::state::AppState {};

  // ルータ組み立て
  let app = api::router::build_router(state);

  // アドレス組み立て
  let addr = format!("{}:{}", settings.server.host, settings.server.port);

  // 指定したアドレスでTCPリスナー(通信窓口)をバインド
  let listener = tokio::net::TcpListener::bind(&addr)
    .await
    .expect("failed to bind address");

  // デバッグ: SqlLayerの動作確認
  for i in 1..=50 - 2 - 4 {
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
