# EarningsWatch 本設計書 02-types. notifierクレート

> `仮設計書-型定義.md` 5章を元にしている。内容の変更はなく、構成の移設のみ。

## 目次
1. [方針](#1-方針)
2. [型定義](#2-型定義)
3. [対応関係](#3-対応関係)
4. [残課題](#4-残課題次節以降で検討)

---

## 1. 方針

- **`webhook_url`は暗号化状態を型で区別する**。暗号化・復号のロジック自体は「機密情報の保存方式」全般の関心事であるため、`notifier`固有の型としては持たず、**`02-types/crypto.md`で新設した`crypto`クレートの汎用型`Encrypted<T>`/`Plain<T>`を利用する**(`T = crypto::WebhookUrlTag`)。誤って暗号化済み文字列をそのまま送信してしまう事故を型レベルで防ぐ
- **`embed_color`はRGB値を持つ専用型`EmbedColor`にする**。本質はRGBであるため`r`/`g`/`b`の`u8`3値として持ち、DB保存・API送受信時のみ`0x87EB87`形式の文字列に変換する(`to_hex_string`/`from_hex_string`)
- **`mention_targets`は構造体上`Vec<String>`のまま(DB `TEXT[]`に対応)持ち、利用側で`MentionTarget::parse`を都度呼ぶ**。構造体自体を`Vec<MentionTarget>`にすると不正要素混入時に構造体が組み立てられなくなるため、パースは利用側(Discord送信ロジック)に委ねる
- **不正なmention_targets要素は警告ログを残しつつ当該要素のみスキップし、送信自体は続行する**方針とする(1件の不正値のために送信全体を止めない)
- **Slack側(`SlackConfig`)は仮実装として型だけ用意し、中身を空にしておく**。Discordを基準に先行実装し、詳細仕様はMVP内拡張フェーズ(Discord実装完了後)で再定義する(`03-features/notification.md`参照)

## 2. 型定義

```rust
use crypto::{Encrypted, WebhookUrlTag};
use identity::GroupId;
use serde::{Deserialize, Serialize};

// ===== embed_color(RGB本質を型で表現、保存形式は文字列) =====
// 04-security.md: 16進カラーコード文字列(例: 0x87EB87)。フロントは0〜255のRGBスライダーで選択し、
// この文字列形式に変換して送信する。本質はRGBのため、Rust側もRGB値として持ち、
// 保存・送受信時のみ文字列形式に変換する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbedColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl EmbedColor {
    // config管理のデフォルト色(水色 0x87CEEB)
    pub const DEFAULT: Self = Self { r: 0x87, g: 0xCE, b: 0xEB };

    pub fn from_hex_string(s: &str) -> Result<Self, ParseColorError> {
        todo!("\"0x87EB87\"形式の文字列からのパース")
    }

    pub fn to_hex_string(&self) -> String {
        format!("0x{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("embed_colorの形式が不正です")]
pub struct ParseColorError;

// ===== mention_targets(判別Union) =====
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeStyle {
    T,       // t: 4:20 PM(短い時刻)
    LongT,   // T: 4:20:30 PM(長い時刻)
    D,       // d: 3/6/2026(短い日付)
    LongD,   // D: March 6, 2026(長い日付)
    F,       // f: March 6, 2026 4:20 PM(短い日付時刻、Discordデフォルト)
    LongF,   // F: Friday, March 6, 2026 4:20 PM(長い日付時刻)
    R,       // R: in 2 hours(相対時間)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MentionTarget {
    User(String),   // user:<discord_user_id>
    Role(String),   // role:<discord_role_id>
    Everyone,       // everyone
    Here,           // here
    Time(TimeStyle), // time:<style>
}

impl MentionTarget {
    // 不正な文字列は警告ログを残しつつスキップする方針(呼び出し側でErrをログに記録して無視する想定)
    pub fn parse(raw: &str) -> Result<Self, ParseMentionTargetError> {
        todo!("プレフィックス(user:/role:/time:)またはeveryone/hereでの機械的な判別")
    }
}

#[derive(Debug, thiserror::Error)]
#[error("mention_targets要素の形式が不正です: {raw}")]
pub struct ParseMentionTargetError {
    pub raw: String, // 元の不正な文字列(ログ出力用)
}

// ===== Discord固有設定 =====
// DB: notify_discord_configs(01-db-schema.md 4章)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub group_id: GroupId,
    pub webhook_url: Option<Encrypted<WebhookUrlTag>>, // NULL許容(未設定時は送信を試みない)
    pub embed_color: Option<EmbedColor>,          // NULLならデフォルト色として判定
    pub mention_enabled: bool,
    pub mention_targets: Vec<String>, // DB上はTEXT[]のまま。MentionTarget::parseで都度パースして使う
}

// ===== Slack固有設定(仮実装、詳細未定) =====
// DB: notify_slack_configs(01-db-schema.md 4章)。フィールド構成はDiscordに準じた仮カラムのみ。
// Slack Incoming Webhook / Block Kitの仕様確認後、MVP内拡張フェーズで再定義する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub group_id: GroupId,
    pub webhook_url: Option<Encrypted<WebhookUrlTag>>,
    pub mention_enabled: bool,
    pub mention_targets: Vec<String>, // Slack側の記法・変換ロジックは未定
}
```

## 3. 対応関係

| 型 | 対応セクション | 備考 |
|---|---|---|
| `Encrypted<WebhookUrlTag>` / `Plain<WebhookUrlTag>` | `02-types/crypto.md`、`04-security.md` | 実体は`crypto`クレート、暗号化状態を型で区別 |
| `EmbedColor` | `04-security.md` | RGB値として保持、文字列変換は`to_hex_string`/`from_hex_string` |
| `TimeStyle` | `03-features/notification.md` | 7種(t/T/d/D/f/F/R) |
| `MentionTarget` | `01-db-schema.md` 4章、`03-features/notification.md` | 判別Union、パースは利用側が呼ぶ |
| `DiscordConfig` | `01-db-schema.md` 4章 | `notify_discord_configs`に対応 |
| `SlackConfig` | `01-db-schema.md` 4章、`03-features/notification.md` | 仮実装。詳細はMVP内拡張フェーズで再定義 |

## 4. 残課題(次節以降で検討)

- `Encrypted::decrypt`・`EmbedColor::from_hex_string`・`MentionTarget::parse`の実装本体(シグネチャのみ確定)
- 不正な`mention_targets`要素検知時の警告ログの具体的な発行箇所・フォーマット
- `SlackConfig`の本格的な型定義(Discord実装完了後のMVP内拡張フェーズ)
