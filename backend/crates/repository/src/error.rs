/*
backend/crates/repository/src/error.rs
repositoryクレートのエラー型の定義
*/

// 外部クレート
// エラー型作成用
use thiserror::Error;

/// repositoryクレートのエラー型
#[derive(Debug, Error)]
pub enum RepositoryError {
  #[error("対象が見つかりません")]
  NotFound,
  #[error("一意制約違反です")]
  Conflict,
  #[error("永続化層でエラーが発生しました: {0}")]
  Other(String),
}

/// repositoryクレートのリザルト
pub type RepositoryResult<T> = Result<T, RepositoryError>;
