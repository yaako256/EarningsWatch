// crates/cli/src/main.rs
//! Phase 3時点では、サブコマンド分岐(create-admin/migration/monitor/notify)を持たず、
//! 起動すると無条件でマイグレーションを実行するだけの素朴な実装とする。
//! Phase 6でclapによるサブコマンド分岐を導入し、このロジックは`migration`サブコマンドの
//! ハンドラ関数としてそのまま再利用する(本書3.3節参照)。

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
  let settings = config::load().expect("failed to load config");

  let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&settings.database.url)
    .await
    .expect("failed to connect to database");

  // sqlx::migrate!マクロは、cliクレートのCargo.tomlがあるディレクトリ
  // (backend/crates/cli/)からの相対パスで migrations/ を探す。
  // 実体は backend/migrations/ のため "../../migrations" を指定する。
  sqlx::migrate!("../../migrations")
    .run(&pool)
    .await
    .expect("failed to run migrations");

  println!("migrations applied successfully");
}
