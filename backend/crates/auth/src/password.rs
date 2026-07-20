/*
backend/crates/auth/src/password.rs
パスワードのハッシュ化などを定義
*/

// 外部クレート
use argon2::Argon2;
use argon2::password_hash::{
  PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString, rand_core::OsRng,
};

/// パスワードをargon2でハッシュ化。
pub fn hash_password(plain: &str) -> Result<String, PasswordHashError> {
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();

  let hash = argon2
    .hash_password(plain.as_bytes(), &salt)
    .map_err(|_| PasswordHashError::HashFailed)?;

  Ok(hash.to_string())
}

/// ログイン時のパスワード検証。
pub fn verify_password(plain: &str, hash: &str) -> Result<bool, PasswordHashError> {
  let parsed_hash = PasswordHash::new(hash).map_err(|_| PasswordHashError::HashFailed)?;
  Ok(
    Argon2::default()
      .verify_password(plain.as_bytes(), &parsed_hash)
      .is_ok(),
  )
}

/// パスワード強度バリデーション8文字以上のみ)。
pub fn validate_password_strength(plain: &str) -> Result<(), PasswordValidationError> {
  if plain.chars().count() < 8 {
    return Err(PasswordValidationError::TooShort);
  }
  Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum PasswordHashError {
  #[error("パスワードのハッシュ化に失敗しました")]
  HashFailed,
}

#[derive(Debug, thiserror::Error)]
pub enum PasswordValidationError {
  #[error("パスワードは8文字以上で入力してください")]
  TooShort,
}
