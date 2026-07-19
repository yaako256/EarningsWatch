// crates/app/src/lib.rs
//! appクレート。ユースケース層(ログイン、フィルタ管理、CSVインポート、monitor/notify実行フロー等)。
//! HTTP・axum・SQL・PostgreSQLには依存しない。AppError/AppResultの定義含め、実装本体はPhase 4以降で行う。
