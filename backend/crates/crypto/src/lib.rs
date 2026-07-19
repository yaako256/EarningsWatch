/*
backend/crates/crypto/src/lib.rs
cryptoクレートでは暗号化系の値の共通型を定義する
*/

mod error;
mod models;

pub use error::DecryptError;
pub use models::*;

// ===== 用途タグ(型パラメータとして使うマーカー型、フィールドは持たない) =====
// タグ生成用マクロ
// "#[derive～]"の設定の重複防止目的
macro_rules! define_crypto_tag {
  ($name:ident) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct $name;
  };
}
define_crypto_tag!(WebhookUrlTag);
define_crypto_tag!(SystemNotifyWebhookUrlTag);
