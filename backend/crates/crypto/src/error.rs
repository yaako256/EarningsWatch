/*
backend/crates/crypto/src/error.rs
cryptoクレートのエラー型の定義
*/

// cryptoクレートのエラー型
#[derive(Debug, thiserror::Error)]
pub enum DecryptError {
  #[error("復号に失敗しました")]
  Failed,
}

/// cryptoクレートのリザルト
pub(crate) type DecryptResult<T> = Result<T, DecryptError>;
