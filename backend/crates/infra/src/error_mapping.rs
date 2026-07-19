/*
backend/crates/infra/src/error_mapping.rs
エラー型の定義
*/

// 内部ライブラリ
use repository::RepositoryError;

/// UNIQUE制約違反(PostgreSQLエラーコード23505)をConflictへ、それ以外をOtherへマッピングする。
/// insert系メソッドで共通に使う。
pub fn map_conflict_error(e: sqlx::Error) -> RepositoryError {
  if let sqlx::Error::Database(db_err) = &e {
    if db_err.code().as_deref() == Some("23505") {
      return RepositoryError::Conflict;
    }
  }
  RepositoryError::Other(e.to_string())
}

/// それ以外の一般的なエラー変換(SELECT/UPDATE/DELETE等で使う)。
pub fn map_error(e: sqlx::Error) -> RepositoryError {
  RepositoryError::Other(e.to_string())
}
