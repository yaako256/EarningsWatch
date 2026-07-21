/*
backend/crates/crypto/src/error.rs
cryptoクレートのエラー型の定義
*/

// cryptoクレートのエラー型
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
  #[error("鍵の形式が不正です")]
  InvalidKey,
  #[error("暗号化に失敗しました")]
  EncryptFailed,
  #[error("復号に失敗しました(改ざん、または対象違いの可能性があります)")]
  DecryptFailed,
  #[error("暗号文の形式が不正です")]
  InvalidCiphertext,
}

/// cryptoクレートのリザルト
pub(crate) type CryptoResult<T> = Result<T, CryptoError>;
