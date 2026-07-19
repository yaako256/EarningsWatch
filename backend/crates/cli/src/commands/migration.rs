/*
backend/crates/cli/src/commands/migration.rs
migrationを実行するコマンド
*/
use sqlx::PgPool;

pub async fn run(pool: &PgPool) {
  match sqlx::migrate!("../../migrations").run(pool).await {
    Ok(()) => println!("migrations applied successfully"),
    Err(e) => {
      eprintln!("マイグレーションの適用に失敗しました: {e}");
      std::process::exit(1);
    }
  }
}
