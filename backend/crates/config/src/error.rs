/*
backend/crates/config/src/error.rs
configクレートのエラー型の定義
*/

// 標準ライブラリ
use std::env;

// 外部クレート
// エラー型作成用
use thiserror::Error;
// config::ConfigError型用
use config;

/// configクレートのエラー型
#[derive(Debug, Error)]
pub enum ConfigLoadError {
  #[error("環境変数の取得失敗: {0}")]
  Var(#[from] env::VarError),
  #[error("設定の読み込みに失敗しました: {0}")]
  Build(config::ConfigError),
  #[error("設定のデシリアライズに失敗しました: {0}")]
  Deserialize(config::ConfigError),
}

/// configクレートのリザルト
pub(crate) type ConfigLoadResult<T> = Result<T, ConfigLoadError>;
