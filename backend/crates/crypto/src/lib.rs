/*
backend/crates/crypto/src/lib.rs
cryptoクレートでは暗号化系の値の共通型を定義する
*/

mod error;
mod models;

pub use error::DecryptError;
pub use models::*;

// ===== 用途タグ(型パラメータとして使うマーカー型、フィールドは持たない) =====
pub struct WebhookUrlTag;
pub struct SystemNotifyWebhookUrlTag;
