/*
backend/crates/crypto/src/models.rs
暗号化系の共通型定義
*/

// 自クレート
use crate::error::DecryptResult;

// 外部クレート
use serde::{Deserialize, Serialize};

// 暗号化済み文字列型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encrypted<T> {
  ciphertext: String,
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

  // AADにはgroup_id等の識別子を渡す想定(04-security.md「暗号文が別グループのレコードへ転用されても検知できるように」)
  pub fn decrypt(&self, _key: &[u8], _aad: &[u8]) -> DecryptResult<Plain<T>> {
    todo!("AES-256-GCMでの復号処理(design/04-security.md参照、実装はPhase 5以降)")
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
