# EarningsWatch 本設計書 02-types. earningsクレート

> `仮設計書-型定義.md` 3章を元にしている。内容の変更はなく、構成の移設のみ。

## 目次
1. [方針](#1-方針)
2. [型定義](#2-型定義)
3. [対応関係](#3-対応関係)
4. [残課題](#4-残課題次節以降で検討)

---

## 1. 方針

- タイムゾーン方針に従い、`fetched_at`(スクレイピング開始時刻)・`published_at`(スクレイピング元サイト表示時刻、JST解釈→UTC変換済み)は共に`DateTime<Utc>`とする(仮設計書ver1の`DateTime<FixedOffset>`から変更)
- `source`は`01-db-schema.md` 5章で`earnings_source` enum(現状`'kabuyoho'`のみ)が確定済みのため、Rust側も`String`ではなく`EarningsSource` enumとする(将来サイト追加時にvariantを追加する運用)
- **DB保存前後で型を分ける**。理由:
  - DB保存前(`Earnings`)は`id`(自動採番)・`fingerprint`(6.4節相当の正規化関数による計算結果)がまだ存在しない
  - `fingerprint`は`Earnings`から計算される側の値であり、フィールドとして`Option`で持たせるのは歪なため、DB保存後専用の型(`EarningsRecord`)を別途用意する
- `camelCase`変換(`#[serde(rename_all = "camelCase")]`)は付与しない。`earnings`クレートはHTTP非依存のドメイン層のため、JSONのcamelCase変換は`api`クレートのDTO層(`02-types/api.md`)で行う

## 2. 型定義

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ===== 決算評価 =====
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "earnings_evaluation", rename_all = "UPPERCASE")]
pub enum EarningsEvaluation {
    Positive,
    Neutral,
    Negative,
    Unrated,
}

// ===== 取得元サイト =====
// 新しいスクレイピング対象サイトを追加する場合はここにvariantを追加する(01-db-schema.md 5章)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "earnings_source", rename_all = "lowercase")]
pub enum EarningsSource {
    Kabuyoho,
}

// ===== スクレイピング直後(DB保存前)の決算情報 =====
// fingerprintはこの構造体から正規化関数によって計算される側であり、
// フィールドとしては持たない
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Earnings {
    pub ticker: String,
    pub company_name: String,
    pub published_at: DateTime<Utc>, // JST解釈→UTC変換済み(スクレイピング元サイト表示時刻)
    pub title: String,
    pub url: String,
    pub summary: String,
    pub evaluation: EarningsEvaluation,
}

// ===== スクレイピング結果全体(サイト共通、Python連携の受け渡し単位) =====
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredEarningsReport {
    pub schema_version: u32,
    pub source: EarningsSource,
    pub fetched_at: DateTime<Utc>, // スクレイピング開始時刻(Rust側monitorが記録する時刻のためJST解釈は不要)
    pub items: Vec<Earnings>,
}

// ===== DB保存後の決算情報(earningsテーブルの1行に対応) =====
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsRecord {
    pub id: i64, // BIGINT自動採番(01-db-schema.md 5章)
    pub ticker: String,
    pub company_name: String,
    pub published_at: DateTime<Utc>,
    pub title: String,
    pub url: String,
    pub summary: String,
    pub evaluation: EarningsEvaluation,
    pub fingerprint: String, // 正規化・ハッシュ化(SHA-256)後の値、UNIQUE
    pub source: EarningsSource,
}
```

## 3. 対応関係

| 型 | 対応セクション | 備考 |
|---|---|---|
| `EarningsEvaluation` | `00-overview.md`、`01-db-schema.md` 5章 | DB全大文字・Rustパスカルケースでマッピング |
| `EarningsSource` | `01-db-schema.md` 5章 | DB小文字(`'kabuyoho'`)・Rustパスカルケースでマッピング |
| `Earnings` | `03-features/scraping.md` | DB保存前(スクレイピング直後)専用 |
| `MonitoredEarningsReport` | `03-features/scraping.md` | Python連携の受け渡し単位(サイト共通) |
| `EarningsRecord` | `01-db-schema.md` 5章 | DB保存後(`earnings`テーブルの1行)専用、`id`/`fingerprint`を持つ |

## 4. 残課題(次節以降で検討)

- `compute_fingerprint`(仮称、fingerprintをRust側の1つの関数に集約する方針)のシグネチャ・置き場所(モジュール名)は未確定
- `ticker`正規化関数(接尾辞除去)の置き場所・シグネチャも同様に未確定
- いずれも実装着手時(Phase 4「domain型 / repository trait」)に決定する
