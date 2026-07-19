/*
backend/crates/cli/src/main.rs
CLIのエントリポイント
*/

// 外部クレート
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
  /// マイグレーションを適用する(design/00-overview.md 7.1章)
  Migration,
  /// 決算情報を収集する(design/00-overview.md 6.1章、Phase 11で実装)
  Monitor,
  /// 通知を送信する(design/00-overview.md 6.1章、Phase 11で実装)
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
      eprintln!("monitor コマンドは未実装です(Phase 11で実装予定)");
      std::process::exit(1);
    }
    Command::Notify => {
      eprintln!("notify コマンドは未実装です(Phase 11で実装予定)");
      std::process::exit(1);
    }
  }
}
