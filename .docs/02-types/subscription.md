# EarningsWatch 本設計書 02-types. subscriptionクレート

> `仮設計書-型定義.md` 4章を元にしている。**`NotifyHistoryEntry.group_id`のみ、本設計書作成時の`notify_history.group_id`の`ON DELETE SET NULL`化(`01-db-schema.md` 7章)を反映して`Option<GroupId>`に変更した**(型定義書時点では`ON DELETE CASCADE`前提の`GroupId`だった)。それ以外は内容の変更なく、構成の移設のみ。

## 目次
1. [方針](#1-方針)
2. [型定義](#2-型定義)
3. [対応関係](#3-対応関係)
4. [残課題](#4-残課題次節以降で検討)

---

## 1. 方針

- 対象は`NotifyGroup`(`01-db-schema.md` 4章`notify_groups`)、`NotifyFilter`(同`notify_filters`)、`NotifyMedium`(同`notify_medium` enum)
- **`NotifyMedium`は`subscription`クレートに置く**。「このグループがどの媒体を使うか」はグループ管理の業務ルール(`subscription`の責務)であり、送信処理の詳細を扱う`notifier`とはレイヤーが異なるため
  - 検討過程で「`notifier`側に置く案」も出たが、`app -> notifier, subscription`が独立している依存関係とも整合するため`subscription`側に確定
- **媒体固有設定(`notify_discord_configs`/`notify_slack_configs`)、および`MentionTarget`は`subscription`ではなく`notifier`クレート側で扱う**(`02-types/notifier.md`)
  - 理由: `notifier`は「通知媒体のTraitと媒体ごとの実装、`discord.rs`/`slack.rs`のようにモジュールを追加するだけで媒体を追加できる」クレートであり、DBの媒体固有設定テーブルはまさに「媒体ごとの設定」でこの責務に一致する
  - `MentionTarget`(`User`/`Role`/`Everyone`/`Here`/`Time`)もDiscord固有の記法に基づく型で、`allowed_mentions`組み立てロジックと不可分なため、`notifier`側が自然
- `paused_at`は`Option<DateTime<Utc>>`のままシンプルに持ち、`is_paused()`を判定用の薄いヘルパーとして追加する(YaakoDrive踏襲。専用ラッパー型は作らない)

## 2. 型定義

```rust
use chrono::{DateTime, Utc};
use earnings::{EarningsEvaluation, EarningsSource};
use identity::{FilterId, GroupId, UserId};
use serde::{Deserialize, Serialize};

// ===== 通知媒体 =====
// DB: notify_medium enum('discord', 'slack')(01-db-schema.md 4章)
// NOTE: 媒体固有設定(notify_discord_configs/notify_slack_configs)とMentionTargetは
//       notifierクレート側で扱う(subscriptionには置かない、要議論の経緯あり)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notify_medium", rename_all = "lowercase")]
pub enum NotifyMedium {
    Discord,
    Slack,
}

// ===== 通知グループ =====
// DB: notify_groups(01-db-schema.md 4章)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyGroup {
    pub id: GroupId,
    pub user_id: UserId,
    pub name: String,
    pub medium: NotifyMedium,
    pub paused_at: Option<DateTime<Utc>>, // NULL = アクティブ(YaakoDrive踏襲、Optionのまま素直に持つ)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NotifyGroup {
    pub fn is_paused(&self) -> bool {
        self.paused_at.is_some()
    }
}

// ===== 通知フィルタ =====
// DB: notify_filters(01-db-schema.md 4章)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyFilter {
    pub id: FilterId,
    pub group_id: GroupId,
    pub ticker: String,
    pub company_name: String,
    pub notes: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

// ===== 送信状況・送信履歴 =====
// DB: notify_status enum('ready'|'sent'|'failed')(01-db-schema.md 6章)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notify_status", rename_all = "lowercase")]
pub enum NotifyStatus {
    Ready,
    Sent,
    Failed,
}

// DB: notify_queue(01-db-schema.md 6章、is_monitor_marker列・fingerprint NULL許容化込み)
// is_monitor_marker=trueの行(健全性チェック用マーカー)はこの型では表現しない。
// 決算データ行(is_monitor_marker=false)のみをこの型で表現する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyQueueEntry {
    pub id: i64,
    pub fingerprint: String, // マーカー行ではNULLだが、この型はデータ行専用のためNULL非許容
    pub source: EarningsSource,
    pub fetched_at: DateTime<Utc>,
    pub ticker: String,
    pub company_name: String,
    pub published_at: DateTime<Utc>,
    pub title: String,
    pub url: String,
    pub summary: String,
    pub evaluation: EarningsEvaluation,
    pub status: NotifyStatus,
}

// DB: notify_history(01-db-schema.md 7章)
// NOTE(本設計書での変更): group_idはON DELETE SET NULL化(01-db-schema.md 7章)に伴い
// Option<GroupId>とした(型定義書時点ではON DELETE CASCADE前提のGroupId非Optionだった)。
// グループが削除された後の送信履歴行はgroup_id = Noneとなる。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyHistoryEntry {
    pub id: i64,
    pub group_id: Option<GroupId>,
    pub fingerprint: String,
    pub sent_at: DateTime<Utc>,
    pub status: NotifyStatus,
}

// ===== ユーザ個人設定 =====
// DB: user_settings(01-db-schema.md 4章)
// NOTE: 「通知購読」概念とは直接関係しないが、新規クレートを増やさない方針のもと、
//       user_idをキーに持つ「ユーザに紐づく設定値」という点でsubscriptionクレートに配置する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub user_id: UserId,
    pub memo: Option<String>,
    pub updated_at: DateTime<Utc>,
}
```

## 3. 対応関係

| 型 | 対応セクション | 備考 |
|---|---|---|
| `NotifyMedium` | `01-db-schema.md` 4章 | DB小文字(`'discord'`/`'slack'`)・Rustパスカルケースでマッピング |
| `NotifyGroup` | `01-db-schema.md` 4章 | `is_paused()`ヘルパー付き |
| `NotifyFilter` | `01-db-schema.md` 4章 | 重複行許容(UNIQUE制約なし)のため`id`で一意に扱う |
| `NotifyStatus` | `01-db-schema.md` 6・7章 | `notify_queue`/`notify_history`共通 |
| `NotifyQueueEntry` | `01-db-schema.md` 6章 | `is_monitor_marker=false`の行のみ表現。マーカー行はこの型で扱わない |
| `NotifyHistoryEntry` | `01-db-schema.md` 7章 | `notify_history`の1行に対応。`group_id`は`Option<GroupId>`(本設計書での変更点) |
| `UserSettings` | `01-db-schema.md` 4章 | 「通知購読」とは無関係だが新規クレート回避のためここに配置 |

## 4. 残課題(次節以降で検討)

- 媒体固有設定(`notify_discord_configs`/`notify_slack_configs`)、`MentionTarget`、`TimeStyle`は`02-types/notifier.md`で定義する
- 依存関係(`00-overview.md` 6.4節)には`subscription -> earnings`が反映済み(`NotifyQueueEntry`/`NotifyHistoryEntry`が`earnings`由来の型を持つため)。`earnings -> identity`のみだった上流依存の流れは崩さない向き(`subscription`が`earnings`を利用する側)
