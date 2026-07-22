/*
backend/crates/cli/src/main.rs
CLIのエントリポイント
*/

// 外部クレート
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use clap::{Parser, Subcommand};

// 自クレート
mod commands;

#[derive(Parser)]
#[command(name = "cli", about = "EarningsWatch管理・運用コマンド")]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  /// 管理者ユーザを作成する
  CreateAdmin {
    #[arg(long)]
    username: String,
  },
  /// マイグレーションを適用する
  Migration,
  /// 決算情報を収集する
  Monitor,
  /// 通知を送信する
  Notify,
}

#[tokio::main]
async fn main() {
  let cli = Cli::parse();

  let settings = config::load().expect("failed to load config");

  let pool = infra::create_pool(&settings.database.url)
    .await
    .expect("failed to connect to database");

  match cli.command {
    Command::Migration => commands::migration::run(&pool).await,
    Command::CreateAdmin { username } => commands::create_admin::run(&pool, username).await,
    Command::Monitor => {
      commands::monitor::run(&pool, settings.scraping.recent_fingerprint_limit).await;
      std::process::exit(1);
    }
    Command::Notify => {
      let webhook_enc_key = STANDARD
        .decode(&settings.security.webhook_enc_key)
        .expect("webhook_enc_keyのbase64デコードに失敗しました");
      commands::notify::run(&pool, &webhook_enc_key, &settings.retry).await;
    }
  }
}
