// crates/server/src/main.rs
//! serverバイナリ。HTTPサーバの起動とDI組み立て。
//! GET /api/health が動く最小の状態はPhase 2で実装する(design/00-overview.md 6.1章参照)。

fn main() {
  println!("server: not yet implemented (see Phase 2)");

  // デバック: 設定の読み込み
  let aaa = config::load();
  print!("{:#?}", aaa);
}
