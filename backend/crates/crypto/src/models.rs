/*
backend/crates/crypto/src/models.rs
暗号化系の共通型定義
*/

// 外部クレート
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use rand::RngCore;
use serde::{Deserialize, Serialize};

// 自クレート
use crate::error::{CryptoError, CryptoResult};

// 暗号化済み文字列型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encrypted<T> {
  ciphertext: String,
  #[serde(skip)]
  _marker: std::marker::PhantomData<T>,
}

impl<T> Encrypted<T> {
  pub fn from_ciphertext(ciphertext: String) -> Self {
    Self {
      ciphertext,
      _marker: std::marker::PhantomData,
    }
  }

  pub fn as_str(&self) -> &str {
    &self.ciphertext
  }

  /// 平文を暗号化する
  /// nonce(12byte、暗号化ごとにランダム生成)を先頭に付与しbase64化した文字列を保持する
  /// (`nonce || ciphertext`形式)。
  pub fn encrypt(plain: &str, key: &[u8], aad: &[u8]) -> CryptoResult<Self> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::InvalidKey)?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
      .encrypt(
        nonce,
        aes_gcm::aead::Payload {
          msg: plain.as_bytes(),
          aad,
        },
      )
      .map_err(|_| CryptoError::EncryptFailed)?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(Self::from_ciphertext(STANDARD.encode(combined)))
  }

  /// 復号処理
  /// AAD(group_id等)が暗号化時と異なる場合、GCM認証エラーとして`CryptoError::DecryptFailed`になる
  pub fn decrypt(&self, key: &[u8], aad: &[u8]) -> CryptoResult<Plain<T>> {
    let combined = STANDARD
      .decode(&self.ciphertext)
      .map_err(|_| CryptoError::InvalidCiphertext)?;
    if combined.len() < 12 {
      return Err(CryptoError::InvalidCiphertext);
    }
    let (nonce_bytes, ciphertext) = combined.split_at(12);

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::InvalidKey)?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let plain_bytes = cipher
      .decrypt(
        nonce,
        aes_gcm::aead::Payload {
          msg: ciphertext,
          aad,
        },
      )
      .map_err(|_| CryptoError::DecryptFailed)?;

    let plain = String::from_utf8(plain_bytes).map_err(|_| CryptoError::InvalidCiphertext)?;
    Ok(Plain {
      value: plain,
      _marker: std::marker::PhantomData,
    })
  }
}

// 復号済み(平文)のマーカー型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plain<T> {
  value: String,
  _marker: std::marker::PhantomData<T>,
}

impl<T> Plain<T> {
  pub fn as_str(&self) -> &str {
    &self.value
  }
}
