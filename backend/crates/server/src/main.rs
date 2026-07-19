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

  tracing::info!("Starting EarningWatch server");
  tracing::info!(addr, "server starting");

  // デバッグ: SqlLayerの動作確認
  for i in 1..=48 {
    tracing::info!(index = i, "SqlLayer test");
  }

  // Axumサーバーを起動してリクエストの待機を開始
  axum::serve(listener, app).await.expect("server error");
}
