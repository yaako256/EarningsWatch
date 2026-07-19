/*
backend/crates/auth/src/password.rs
パスワードのハッシュ化などを定義
*/
use argon2::Argon2;
use argon2::password_hash::{PasswordHasher as _, SaltString, rand_core::OsRng};

/// パスワードをargon2でハッシュ化。
/// ログイン時の検証(verify_password)・強度バリデーションはPhase 7で追加する。
pub fn hash_password(plain: &str) -> Result<String, PasswordHashError> {
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();

  let hash = argon2
    .hash_password(plain.as_bytes(), &salt)
    .map_err(|_| PasswordHashError::HashFailed)?;

  Ok(hash.to_string())
}

#[derive(Debug, thiserror::Error)]
pub enum PasswordHashError {
  #[error("パスワードのハッシュ化に失敗しました")]
  HashFailed,
}
