/*
backend/crates/auth/src/refresh_token_issuer.rs
リフレッシュトークンの生成とHash化
*/
use rand::RngCore;
use sha2::{Digest, Sha256};

/// 不透明なリフレッシュトークン(32byteの乱数を16進文字列化)を生成する。
pub fn generate_refresh_token_plain() -> String {
  let mut bytes = [0u8; 32];
  rand::thread_rng().fill_bytes(&mut bytes);
  bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// DB保存用のハッシュ化
/// (パスワードと違い、リフレッシュトークンの照合には高速なSHA-256で十分と判断)
/// (乱数のエントロピーが十分に大きいためargon2のような低速ハッシュによる総当たり耐性は不要)
pub fn hash_refresh_token(plain: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(plain.as_bytes());
  format!("{:x}", hasher.finalize())
}
