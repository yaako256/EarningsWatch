# EarningsWatch 本設計書 02-types. cryptoクレート(新設)

> `仮設計書-型定義.md` 2章を元にしている。内容の変更はなく、構成の移設のみ。

## 目次
1. [経緯・方針](#1-経緯方針)
2. [型定義](#2-型定義)
3. [残課題](#3-残課題次節以降で検討)

---

## 1. 経緯・方針

管理者向け通知先設定(`system_notify_config`、`01-db-schema.md` 9章)でも`webhook_url`の暗号化が必要になったため、当初`notifier`クレート固有だった`EncryptedWebhookUrl`/`PlainWebhookUrl`を汎用化し、新設の`crypto`クレートに切り出した。

- 暗号化・復号のロジック自体は「機密情報の保存方式」全般の関心事であり、`notifier`固有の関心事ではないため独立したクレートとする
- 依存関係(`00-overview.md` 6.4節): `crypto -> 外部crateのみ`、`notifier -> crypto`、`subscription -> crypto`
- ジェネリクスの型パラメータ`T`を「用途タグ」(フィールドを持たないマーカー型、例:`WebhookUrlTag`)として使うことで、異なる用途の暗号化文字列を型レベルで混同できないようにする
- 暗号化方式自体(AES-256-GCM、`nonce || ciphertext`のbase64、`04-security.md`参照)は変更なし。型の置き場所と抽象化レベルのみを汎用化したもの

## 2. 型定義

```rust
use serde::{Deserialize, Serialize};

// 暗号化済み文字列(DB保存形式、nonce || ciphertextのbase64、04-security.mdの方式に準拠)
// webhook_url以外の機密情報(将来増える可能性のあるもの)にも汎用的に使う
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encrypted<T> {
    ciphertext: String, // base64(nonce || ciphertext)
    _marker: std::marker::PhantomData<T>,
}

impl<T> Encrypted<T> {
    pub fn from_ciphertext(ciphertext: String) -> Self {
        Self { ciphertext, _marker: std::marker::PhantomData }
    }

    pub fn as_str(&self) -> &str {
        &self.ciphertext
    }

    // AADにはgroup_id等の識別子を渡す想定(04-security.md「暗号文が別グループのレコードへ転用されても検知できるように」)
    pub fn decrypt(&self, key: &[u8], aad: &[u8]) -> Result<Plain<T>, DecryptError> {
        todo!("AES-256-GCMでの復号処理(04-security.md参照)")
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

#[derive(Debug, thiserror::Error)]
pub enum DecryptError {
    #[error("復号に失敗しました")]
    Failed,
}

// ===== 用途タグ(型パラメータとして使うマーカー型、フィールドは持たない) =====
pub struct WebhookUrlTag;              // notify_discord_configs / notify_slack_configs 用
pub struct SystemNotifyWebhookUrlTag;  // system_notify_config 用(将来AADの粒度を分けたい場合に備える)
```

## 3. 残課題(次節以降で検討)

- `Encrypted::decrypt`の実装本体(シグネチャのみ確定)、鍵(`EARNINGSWATCH__SECURITY__WEBHOOK_ENC_KEY`)を`config`クレートからどう受け渡すかは実装着手時に決定する
