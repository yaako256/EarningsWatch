/*
backend/crates/config/src/error.rs
scraperクレートのエラー型の定義
*/

// 外部クレート
// エラー型作成用
use thiserror::Error;

/// scraperクレートのエラー型
#[derive(Debug, Error)]
pub enum ScraperError {
  #[error("Pythonプロセスの実行に失敗しました: {0}")]
  ProcessFailed(String),
  #[error("Pythonプロセスの出力の解析に失敗しました: {0}")]
  ParseFailed(String),
}

/// scraperクレートのリザルト
pub(crate) type ScraperResult<T> = Result<T, ScraperError>;
