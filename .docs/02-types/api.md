# EarningsWatch 本設計書 02-types. apiクレート

> `仮設計書-型定義.md` 6章(14.5〜14.13節対応)を元にしている。型定義書8章の整合性チェック結果(`handlers/common.rs`収録型一覧・`ExportFormat`の置き場所確定等)を反映済み。
> また、**`NotifyHistoryResponse`は`01-db-schema.md`での`notify_history.group_id`の`ON DELETE SET NULL`化決定を受け、`group_id`/`group_name`を`Option`化する変更を加えている**(型定義書時点では`ON DELETE CASCADE`前提だった)。
> さらに、**`ImportFiltersRequest`/`ImportGroupFiltersRequest`に`dry_run: bool`を追加している**(`03-features/import-export.md`のドライラン機能が型定義書側に未反映のまま持ち越されていたための補完)。それ以外は内容の変更なく、構成の移設のみ。

## 目次
1. [ファイル構造](#1-ファイル構造)
2. [response.rs(エンベロープ形式・エラーコード・ページング)](#2-responsers エンベロープ形式エラーコードページング)
3. [認証API](#3-認証api)
4. [管理者API](#4-管理者api)
5. [決算情報API](#5-決算情報api)
6. [送信履歴API](#6-送信履歴api)
7. [グループAPI](#7-グループapi)
8. [フィルタAPI(一括インポート含む)](#8-フィルタapi一括インポート含む)
9. [ユーザ設定API](#9-ユーザ設定api)
10. [ダッシュボードAPI](#10-ダッシュボードapiユーザ単位)
11. [お知らせ板・固定ページAPI](#11-お知らせ板固定ページapi)
12. [handlers/common.rs 収録型 一覧(正)](#12-handlerscommonrs-収録型-一覧正)
13. [命名・パターンの一貫性チェック](#13-命名パターンの一貫性チェック)

---

## 1. ファイル構造

YaakoDriveの構成を参考にしつつ、機能ごとに1対1で対応する形で`handlers/`を分割する。

```text
api/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── router.rs
    ├── state.rs
    ├── cookie.rs
    ├── error.rs
    ├── response.rs       # エンベロープ形式、ApiErrorCode、Page<T>
    ├── extractor.rs       # JWT認証ミドルウェア等
    └── handlers/
        ├── mod.rs
        ├── common.rs      # 複数エンドポイントで共有するDTO(GroupResponse等、12章に一覧)
        ├── health.rs      # ヘルスチェック、リクエストDTOなし
        ├── auth.rs        # LoginRequest等
        ├── admin.rs       # CreateUserRequest等
        ├── earnings.rs    # EarningsQuery等
        ├── notify_history.rs
        ├── group.rs       # CreateGroupRequest等
        ├── filter.rs      # CreateFilterRequest等
        ├── user_settings.rs
        ├── dashboard.rs
        └── page.rs        # お知らせ板・固定ページ
```

**DTOの置き場所の方針(YaakoDrive踏襲)**
- **リクエストDTO**は該当ハンドラファイル内に直接定義する(1エンドポイントでしか使わないため、ハンドラのすぐ近くにある方が見通しが良い)
- **レスポンスDTO**は複数エンドポイントで使い回されることが多いため、`handlers/common.rs`に集約する(12章に最終一覧)

**ヘルスチェックAPIについて**: `GET /api/health`はリクエストDTO・レスポンスDTOともに専用の型を持たない単純なエンドポイントである。既存の`ApiResponse<()>`(2章)をそのまま使い、`ApiResponse::ok(())`を返すのみで足りるため、本書でも独立した節を設けない。`handlers/health.rs`には、ハンドラ関数本体のみが置かれる想定。

## 2. response.rs(エンベロープ形式・エラーコード・ページング)

- エラーコード(固定10種)は文字列(`&str`)ではなく**`ApiErrorCode` enum**として定義する。タイプミスをコンパイル時に検知でき、HTTPステータスコードとの対応(`status_code()`)もコード内に集約できるため
- `Page<T>`には`#[serde(rename_all = "camelCase")]`を適用する(DTO命名規則)

```rust
use axum::http::StatusCode;
use serde::Serialize;

// ===== エンベロープ形式 =====
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }
}

// エラーはRustがdata型を知る必要がないため、Tなしの別implで定義(YaakoDrive踏襲)
impl ApiResponse<()> {
    pub fn err(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self {
            data: None,
            error: Some(ApiError {
                code,
                message: message.into(),
            }),
        }
    }
}

// ===== エラー本体 =====
#[derive(Serialize)]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub message: String,
}

// ===== エラーコード(固定10種) =====
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiErrorCode {
    Unauthorized,        // 401 未認証
    Forbidden,           // 403 権限不足
    NotFound,            // 404 リソースが存在しない
    AlreadyExists,       // 409 重複(ユーザ名など)
    InvalidRequest,      // 422 リクエスト内容が不正
    NotifyConfigMissing, // 422 送信先設定(webhook_url等)が未設定
    NotifySendFailed,    // 502 送信先への通信自体が失敗(タイムアウト・DNS等)
    NotifyRejected,      // 502 送信先が非2xxを返した
    ImportEmpty,         // 422 インポート対象の行が1件もない(グループ単位インポート時)
    InternalError,       // 500 サーバ内部エラー
}

impl ApiErrorCode {
    pub fn status_code(self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::AlreadyExists => StatusCode::CONFLICT,
            Self::InvalidRequest => StatusCode::UNPROCESSABLE_ENTITY,
            Self::NotifyConfigMissing => StatusCode::UNPROCESSABLE_ENTITY,
            Self::NotifySendFailed => StatusCode::BAD_GATEWAY,
            Self::NotifyRejected => StatusCode::BAD_GATEWAY,
            Self::ImportEmpty => StatusCode::UNPROCESSABLE_ENTITY,
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// ===== ページング共通仕様 =====
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T: Serialize> {
    pub items: Vec<T>,
    pub page: u32,
    pub per_page: u32,
    pub total_count: i64,
    pub total_pages: u32,
}
```

対応関係: `ApiResponse<T>`/`ApiError`はエンベロープ形式、`ApiErrorCode`はHTTPステータス対応表を内包、`Page<T>`は`camelCase`変換あり。

`extractor.rs`(JWT認証ミドルウェア)、`error.rs`(`ApiAppError`から`ApiErrorCode`への変換)の実装本体は実装フェーズで検討する。

## 3. 認証API

### 方針
- **`Role`は`auth`クレートに置く**。JWTの`TokenClaims`・権限判定と一体で扱うべき型のため
- `Role`のDB対応は`users.role`(`TEXT`、CHECK制約なし、値は`admin`/`user`の2値)
- `LoginResponse`と`MeResponse`はあえて共通化せず別の型として定義する(将来の変更影響範囲を分離するため)
- `POST /api/auth/refresh`・`POST /api/auth/logout`はリクエストボディなし(Cookieのみで完結)、レスポンスも`ApiResponse::ok(())`(`data: null`)のため専用DTOは定義しない

### 型定義

```rust
// ===== auth crate側(Role enum) =====
// DB: users.role TEXT('admin'|'user')
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum Role {
    Admin,
    User,
}
```

```rust
// ===== api crate側(handlers/auth.rs) =====
use serde::{Deserialize, Serialize};

// ─── POST /api/auth/login ───
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub username: String,
}

// ─── POST /api/auth/refresh ───
// リクエストボディなし(Cookieのリフレッシュトークンのみで完結)
// レスポンスは ApiResponse::ok(()) (data: null)

// ─── POST /api/auth/logout ───
// リクエストボディなし
// レスポンスは ApiResponse::ok(()) (data: null)

// ─── GET /api/auth/me ───
#[derive(Serialize)]
pub struct MeResponse {
    pub username: String,
}
```

### 対応関係

| 型 | 置き場所 | 備考 |
|---|---|---|
| `Role` | `auth`クレート | DB `TEXT`カラムに対応(専用enum型ではない) |
| `LoginRequest` / `LoginResponse` | `api`クレート(`handlers/auth.rs`) | `MeResponse`とは共通化しない |
| `MeResponse` | `api`クレート(`handlers/auth.rs`) | `LoginResponse`とは共通化しない |

---

## 4. 管理者API

### 経緯・方針

- **`logs`テーブルのドメイン型(`LogLevel`/`LogProcess`/`LogEntry`)は`logging`クレートに置く**。ログという概念自体が`logging`クレートの責務であるため
- **`GET /api/admin/logs`はレベル・プロセスでのフィルタも受け付ける**(`ListLogsQuery`に`level`/`process`を追加)。`idx_logs_process`インデックス(`01-db-schema.md` 1章)が用意されている実態に合わせた
- **`LogResponse`には`fields`(JSONB、構造化ログの中身)も含める**。フロントエンドに生のJSONとしてそのまま渡し、表示の作り込みはフロント側に委ねる方針
- **`POST /api/admin/users`(仮ユーザ作成)のレスポンスは、一覧用の`AdminUserResponse`とは意図的に別の型(`CreateUserResponse`)にする**。「生成された仮パスワードは管理者画面に一度だけ表示(再表示不可)」のため、平文パスワードを含む特別なレスポンスをこの1回限りで返す
- **`GET /api/admin/users/{id}/summary`は集計専用DTO**。DBの特定テーブル1行ではなく、`subscription`クレートの`NotifyGroup`/`NotifyFilter`を集計した結果を表す
- **notify-config(定期実行ロガーの通知先設定)は新規テーブル`system_notify_config`として追加している**(`01-db-schema.md` 9章)
  - envで持つ案も検討したが、API設計(`GET`/`PUT`)がすでに「フロントから読み書きできる」ことを前提にしており、DBに寄せる方が手戻りが少ないと判断
  - 当初「`id BOOLEAN PRIMARY KEY DEFAULT TRUE`」で1行のみを保証する設計にしていたが、**管理者が複数人いる場合に設定が上書きされ合う問題**が発覚。今回は**管理者全体で共有する1つの設定**として運用し(UI上で「共通設定」であることを明示)、管理者ごとの個別設定は将来拡張とする(`05-future-work.md`)
- `system_notify_config.webhook_url`は`crypto`クレートの汎用型を用いる。`notify_discord_configs`用の`WebhookUrlTag`とは区別し、専用の`SystemNotifyWebhookUrlTag`を使う(AADの粒度を分けるため)
- `GET /api/admin/dashboard`は管理者ダッシュボードの表(累計スクレイピング件数・送信成功率・最終監視実行時刻・実行時間推移)とそのまま対応する

### 型定義

```rust
// ===== logging crate側 =====
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

// DB: logs.level VARCHAR(5) CHECK(...)(01-db-schema.md 1章)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "UPPERCASE")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

// DB: logs.process log_process enum('server'|'monitor'|'notify')(01-db-schema.md 1章)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "log_process", rename_all = "lowercase")]
pub enum LogProcess {
    Server,
    Monitor,
    Notify,
}

// DB: logsテーブルの1行に対応(01-db-schema.md 1章)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub process: LogProcess,
    pub target: String,
    pub message: Option<String>,
    pub fields: JsonValue, // JSONB(構造化ログ用)
}
```

```rust
// ===== subscription crate側(新規テーブル、管理者全体で共有する通知先設定) =====
use crypto::{Encrypted, SystemNotifyWebhookUrlTag};
use serde::{Deserialize, Serialize};
use crate::NotifyMedium; // notify_groupsと同じenumを再利用

// DB: system_notify_config(01-db-schema.md 9章、管理者全体で共有・1行のみ運用)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemNotifyConfig {
    pub medium: NotifyMedium,
    pub webhook_url: Option<Encrypted<SystemNotifyWebhookUrlTag>>,
    pub mention_enabled: bool,
    pub mention_targets: Vec<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

```rust
// ===== api crate側(handlers/admin.rs) =====
use auth::Role;
use chrono::{DateTime, Utc};
use identity::UserId;
use logging::{LogLevel, LogProcess};
use serde::{Deserialize, Serialize};
use subscription::NotifyMedium;

// ─── GET /api/admin/logs ───
#[derive(Deserialize)]
pub struct ListLogsQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub level: Option<LogLevel>,
    pub process: Option<LogProcess>,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Serialize)]
pub struct LogResponse {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub process: LogProcess,
    pub target: String,
    pub message: Option<String>,
    pub fields: serde_json::Value, // 構造化ログの中身をそのままフロントへ渡す
}

// ─── GET /api/admin/users ───
#[derive(Serialize)]
pub struct AdminUserResponse {
    pub id: UserId,
    pub username: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub disabled_at: Option<DateTime<Utc>>,
}

// ─── POST /api/admin/users(仮ユーザ作成) ───
#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
}

// 一度だけ平文パスワードを含む専用レスポンス(再表示不可)
// AdminUserResponseとは意図的に型を分ける
#[derive(Serialize)]
pub struct CreateUserResponse {
    pub id: UserId,
    pub username: String,
    pub temporary_password: String, // この時点でのみ平文を返す
}

// ─── POST /api/admin/users/{id}/disable ───
// リクエストボディなし、レスポンスは ApiResponse::ok(())

// ─── GET /api/admin/users/{id}/summary ───
// 集計専用DTO。DB特定テーブルの1行ではなくNotifyGroup/NotifyFilterの集計結果
#[derive(Serialize)]
pub struct UserSummaryResponse {
    pub group_count: u32,
    pub filter_count: u32,
    pub discord_group_count: u32,
    pub slack_group_count: u32,
}

// ─── GET/PUT /api/admin/notify-config ───
#[derive(Serialize)]
pub struct NotifyConfigResponse {
    pub medium: NotifyMedium,
    pub webhook_url: Option<String>, // 復号済み平文
    pub mention_enabled: bool,
    pub mention_targets: Vec<String>,
}

#[derive(Deserialize)]
pub struct UpdateNotifyConfigRequest {
    pub medium: NotifyMedium,
    pub webhook_url: Option<String>,
    pub mention_enabled: bool,
    pub mention_targets: Vec<String>,
}

// ─── GET /api/admin/dashboard ───
#[derive(Serialize)]
pub struct AdminDashboardResponse {
    pub total_earnings_count: i64,       // 累計スクレイピング件数
    pub notify_success_rate: f64,        // 送信成功率
    pub last_monitor_run_at: Option<DateTime<Utc>>, // 最終監視実行時刻
    pub run_durations: Vec<SystemRunDuration>,      // 実行時間の推移
}

#[derive(Serialize)]
pub struct SystemRunDuration {
    pub run_type: SystemRunType,
    pub run_at: DateTime<Utc>,
    pub duration_ms: i32,
}

// DB: system_runs.run_type run_type enum('monitor'|'notify')(01-db-schema.md 8章)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "run_type", rename_all = "lowercase")]
pub enum SystemRunType {
    Monitor,
    Notify,
}
```

### 対応関係

| 型 | 置き場所 | 備考 |
|---|---|---|
| `LogLevel` / `LogProcess` / `LogEntry` | `logging`クレート | ログドメインの型 |
| `SystemNotifyConfig` | `subscription`クレート | 管理者全体共有・1行のみ運用 |
| `ListLogsQuery` / `LogResponse` | `api`クレート(`handlers/admin.rs`) | `fields`(JSONB)を含む |
| `AdminUserResponse` | `api`クレート(`handlers/admin.rs`) | ユーザ一覧用 |
| `CreateUserRequest` / `CreateUserResponse` | `api`クレート(`handlers/admin.rs`) | `CreateUserResponse`のみ平文パスワードを含む |
| `UserSummaryResponse` | `api`クレート(`handlers/admin.rs`) | 集計専用DTO |
| `NotifyConfigResponse` / `UpdateNotifyConfigRequest` | `api`クレート(`handlers/admin.rs`) | `system_notify_config`に対応 |
| `AdminDashboardResponse` / `SystemRunDuration` / `SystemRunType` | `api`クレート(`handlers/admin.rs`) | |

### 残課題(次節以降で検討)

- 管理者ごとの個別通知先設定(複数管理者対応)は将来拡張(`05-future-work.md`)として持ち越し

---

## 5. 決算情報API

### 経緯・方針

- **レスポンスは`api`クレート側の専用型(`EarningsResponse`)を作る**。`earnings::EarningsRecord`はHTTP非依存のドメイン層のため、camelCase変換はここでは付与せず、`api`層で変換込みの型を別途用意する
- **`from`/`to`は`published_at`(決算発表日時)のみを対象とする**。`fetched_at`(スクレイピング取得日時)でのフィルタは現時点では設けない
- **`GET /api/earnings/export`はCSVからExcel(xlsx)に変更する**(仕様変更、詳細は下記)
- **`/api/earnings/summary`は決算発表日ごとの件数集計のみ**とする。評価(Positive/Negative等)別の内訳は現時点では持たない
- **summaryの日付は例外的にJST基準で切り出す**。タイムゾーン方針(原則UTCのまま返しJST変換はフロント側の責務、`00-overview.md`)の例外にあたるため、フィールド名を`date_jst`とし、JST基準であることを明示する

### 仕様変更: 決算情報エクスポートをCSVからExcel(xlsx)へ変更

フィルタデータのExcel出力(8章)で決定した経緯(文字コード起因の文字化け、`ticker`のような先頭ゼロを含む値が数値として誤解釈される「列の型崩れ」リスク)は、`ticker`列を持つ決算情報エクスポートにも同様に当てはまるため、`GET /api/earnings/export`もフィルタエクスポートと同じく`format=xlsx`クエリパラメータを持つ形式に変更する。CSV対応は将来拡張(`05-future-work.md`)。

```text
GET /api/earnings/export?format=xlsx  - 決算情報一覧のExcelエクスポート(旧: CSVエクスポート)
```

### 型定義

```rust
use chrono::{DateTime, NaiveDate, Utc};
use earnings::EarningsEvaluation;
use serde::{Deserialize, Serialize};

// ─── GET /api/earnings ───
#[derive(Deserialize)]
pub struct ListEarningsQuery {
    pub ticker: Option<String>,
    pub company_name: Option<String>,
    pub evaluation: Option<EarningsEvaluation>,
    pub from: Option<DateTime<Utc>>, // published_atに対するフィルタ
    pub to: Option<DateTime<Utc>>,   // published_atに対するフィルタ
    pub page: u32,
    pub per_page: u32,
}

// api層専用のレスポンス型(earnings::EarningsRecordはHTTP非依存のためcamelCase変換しない方針)
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsResponse {
    pub id: i64,
    pub ticker: String,
    pub company_name: String,
    pub published_at: DateTime<Utc>,
    pub title: String,
    pub url: String,
    pub summary: String,
    pub evaluation: EarningsEvaluation,
}

impl From<earnings::EarningsRecord> for EarningsResponse {
    fn from(record: earnings::EarningsRecord) -> Self {
        Self {
            id: record.id,
            ticker: record.ticker,
            company_name: record.company_name,
            published_at: record.published_at,
            title: record.title,
            url: record.url,
            summary: record.summary,
            evaluation: record.evaluation,
        }
    }
}

// ─── GET /api/earnings/export ───
// クエリはListEarningsQueryと同じ条件(ticker/company_name/evaluation/from/to)を共有し、
// page/per_page(ページング)のみ持たない。絞り込んだ全件をxlsxとして返す。
// format(ExportFormat)は handlers/common.rs 側の定義を利用する(12章参照)
#[derive(Deserialize)]
pub struct ExportEarningsQuery {
    pub ticker: Option<String>,
    pub company_name: Option<String>,
    pub evaluation: Option<EarningsEvaluation>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub format: crate::handlers::common::ExportFormat, // MVPでは`xlsx`のみ対応
}
// レスポンス本体はJSONではなくxlsxバイナリ
// (Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet)

// ─── GET /api/earnings/summary ───
#[derive(Deserialize)]
pub struct EarningsSummaryQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsSummaryResponse {
    pub daily_counts: Vec<DailyEarningsCount>, // 決算発表日ごとの件数(評価別の内訳は持たない)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyEarningsCount {
    // published_at(UTC)をJSTに変換した上での日付。タイムゾーン方針の例外にあたるため、
    // フィールド名(date_jst -> JSON化後 dateJst)でJST基準であることを明示する。
    pub date_jst: NaiveDate,
    pub count: u32,
}
```

### 対応関係

| 型 | 備考 |
|---|---|
| `ListEarningsQuery` | `from`/`to`は`published_at`基準 |
| `EarningsResponse` | `earnings::EarningsRecord`からの変換専用。`camelCase`変換あり |
| `ExportEarningsQuery` | CSVからxlsxへ変更(仕様変更)。`ExportFormat`は`common.rs`から参照 |
| `EarningsSummaryQuery` / `EarningsSummaryResponse` / `DailyEarningsCount` | 日別集計、評価別内訳なし。`date_jst`はJST基準の例外フィールド |

### 残課題(次節以降で検討)

- xlsx生成処理自体(列構成、ライブラリ選定)は8章のフィルタエクスポートと共通化できる可能性があるが、実装フェーズで検討する
- `DailyEarningsCount`のJST変換をDBクエリ側(`AT TIME ZONE 'Asia/Tokyo'`)で行うか、Rust側で変換するかは未確定

---

## 6. 送信履歴API

### 経緯・方針

- **`notify_queue`/`notify_history`関連のドメイン型(`NotifyStatus`/`NotifyQueueEntry`/`NotifyHistoryEntry`)は`subscription`クレートに置く**(`02-types/subscription.md`)。新規クレートを増やさない方針のもと、「送信管理」という関心事が`subscription`の責務に近いと判断した
- **`is_monitor_marker=true`のマーカー行はAPIレスポンスから除外する**。内部的な健全性チェック用の行であり、ユーザ向けAPIとしては意味を持たないため
- **`GET /api/notify-history`のレスポンスにはグループ名(`group_name`)もJOIN済みで含める**(拡張性重視の方針)。フロント側でグループ名解決のための追加リクエストを不要にする
- **【本設計書での変更】`group_id`/`group_name`を`Option`化した**。`01-db-schema.md`で`notify_history.group_id`を`ON DELETE SET NULL`に確定したため、グループ削除後は`group_id`が`NULL`になり、`group_name`のJOINも`INNER JOIN`ではなく`LEFT JOIN`になる。型定義書時点(`ON DELETE CASCADE`前提)では両方とも非Optionだったが、本設計書ではこれを反映して`Option`化する

### 型定義

```rust
use chrono::{DateTime, Utc};
use identity::GroupId;
use serde::{Deserialize, Serialize};
use subscription::{NotifyHistoryEntry, NotifyQueueEntry, NotifyStatus};

// ─── GET /api/notify-queue ───
#[derive(Deserialize)]
pub struct ListNotifyQueueQuery {
    pub status: Option<NotifyStatus>,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyQueueResponse {
    pub id: i64,
    pub ticker: String,
    pub company_name: String,
    pub published_at: DateTime<Utc>,
    pub title: String,
    pub evaluation: earnings::EarningsEvaluation,
    pub status: NotifyStatus,
}

impl From<NotifyQueueEntry> for NotifyQueueResponse {
    fn from(e: NotifyQueueEntry) -> Self {
        Self {
            id: e.id,
            ticker: e.ticker,
            company_name: e.company_name,
            published_at: e.published_at,
            title: e.title,
            evaluation: e.evaluation,
            status: e.status,
        }
    }
}

// ─── GET /api/notify-history ───
#[derive(Deserialize)]
pub struct ListNotifyHistoryQuery {
    pub group_id: Option<GroupId>, // フィルタ用パラメータ(絞り込み条件、レスポンスのgroup_idとは無関係)
    pub status: Option<NotifyStatus>,
    pub page: u32,
    pub per_page: u32,
}
// レスポンス型 NotifyHistoryResponse は handlers/common.rs に配置する(dashboard.rsからも参照するため)
```

```rust
// ===== handlers/common.rs =====
use chrono::{DateTime, Utc};
use identity::GroupId;
use serde::Serialize;
use subscription::NotifyStatus;

// 拡張性重視: グループ名もJOIN済みで返す(フロント側でグループ名解決の追加リクエストを不要にする)
// group_id/group_nameはOption。グループがON DELETE SET NULLで削除された履歴行では両方Noneになる
// (LEFT JOIN notify_groups ON notify_history.group_id = notify_groups.id)
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyHistoryResponse {
    pub id: i64,
    pub group_id: Option<GroupId>,
    pub group_name: Option<String>, // グループ削除後はNone。フロントは「削除済みグループ」等にフォールバック表示する想定
    pub fingerprint: String,
    pub sent_at: DateTime<Utc>,
    pub status: NotifyStatus,
}
```

### 対応関係

| 型 | 備考 |
|---|---|
| `ListNotifyQueueQuery` / `NotifyQueueResponse` | マーカー行除外、`fingerprint`はレスポンスに含めない |
| `ListNotifyHistoryQuery` | |
| `NotifyHistoryResponse` | `handlers/common.rs`配置(dashboard.rsと共有するため)。`group_id`/`group_name`は`Option`(本設計書での変更点) |

---

## 7. グループAPI

### 経緯・方針

- **`GET/PUT /api/groups/{id}/config`は`medium`(Discord/Slack)によって形が変わるため、判別Union(タグ付き、`#[serde(tag = "medium", ...)]`)で表現する**。`MentionTarget`同様のパターンを踏襲
- **`webhook_url`はAPIレスポンス上は復号済み平文(`String`)で返す**。フロントエンド側は暗号化/復号を一切行わないため
- **`POST /api/groups/{id}/config/test-send`は、HTTPレベルでの送信成否のみを判定する**。「意図した送信先に届いたか」の検証(webhook_url自体の設定ミス等)はユーザの責任範囲とし、システムはHTTP通信・ステータスコードの成否のみをレスポンスする
- **`test-send`のリクエストボディは全フィールド任意で、送信内容を自由入力してプレビューできる**(追加仕様「送信先webhookのテスト送信・プレビュー機能拡張」)。未入力時はバックエンド側でデフォルト値(ダミー値・グループ保存済み設定等)を補完し、`webhook_url`/`mention_targets`を上書きしてもDBには保存しない一時送信とする
- **`PUT /api/groups/bulk-destination`は、対象グループ全てに同一の設定を反映する**設計とする。処理件数(`updated_count`)を8章の一括インポート系と同様の形で返す
- `PATCH /pause`/`resume`は専用レスポンスを作らず、更新後の`GroupResponse`を使い回す

### 型定義

```rust
// ===== handlers/common.rs(複数エンドポイントで共有するレスポンス) =====
use chrono::{DateTime, Utc};
use identity::GroupId;
use serde::{Deserialize, Serialize};
use subscription::{NotifyGroup, NotifyMedium};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupResponse {
    pub id: GroupId,
    pub name: String,
    pub medium: NotifyMedium,
    pub paused_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<NotifyGroup> for GroupResponse {
    fn from(g: NotifyGroup) -> Self {
        Self {
            id: g.id,
            name: g.name,
            medium: g.medium,
            paused_at: g.paused_at,
            created_at: g.created_at,
            updated_at: g.updated_at,
        }
    }
}
```

```rust
// ===== handlers/group.rs =====
use identity::GroupId;
use serde::{Deserialize, Serialize};
use subscription::NotifyMedium;

// ─── GET /api/groups ───
// レスポンスは Vec<GroupResponse>(Page<GroupResponse>ではなく全件、page/per_pageクエリなし)

// ─── POST /api/groups ───
#[derive(Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub medium: NotifyMedium,
}
// レスポンスは GroupResponse

// ─── PUT /api/groups/{id} ───
#[derive(Deserialize)]
pub struct UpdateGroupRequest {
    pub name: String,
    pub medium: NotifyMedium,
}
// レスポンスは GroupResponse

// ─── DELETE /api/groups/{id} ───
// リクエストボディなし、レスポンスは ApiResponse::ok(())

// ─── PATCH /api/groups/{id}/pause, /resume ───
// リクエストボディなし
// レスポンスは GroupResponse(更新後のpaused_atを含めて返す。専用レスポンスは作らない)

// ─── GET/PUT /api/groups/{id}/config ───
// medium(Discord/Slack)によって形が変わるため、判別Union(タグ付き)で表現する
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "medium", rename_all = "lowercase")]
pub enum GroupConfigDto {
    Discord {
        webhook_url: Option<String>, // 復号済み平文(フロントは暗号化/復号を行わない)
        embed_color: Option<String>, // "0x87EB87"形式(EmbedColor::to_hex_string経由)
        mention_enabled: bool,
        mention_targets: Vec<String>,
    },
    Slack {
        // 詳細未定の仮実装(notifier::SlackConfigに準拠)
        webhook_url: Option<String>,
        mention_enabled: bool,
        mention_targets: Vec<String>,
    },
}
// GET: GroupConfigDtoをそのまま返す
// PUT: GroupConfigDtoをそのまま受け取る(リクエスト/レスポンスで同一の型を共有)

// ─── POST /api/groups/{id}/config/test-send ───
// 全フィールド任意。追加仕様「送信先webhookのテスト送信・プレビュー機能拡張」により、
// ユーザが自由入力でプレビュー送信できる。未入力の場合はデフォルト値をバックエンド側で補完する。
// webhook_url/mention_targetsを上書きしてもDBには保存しない(その場限りの一時送信)
#[derive(Deserialize)]
pub struct TestSendRequest {
    pub ticker: Option<String>,                 // 未入力: 固定のダミー値
    pub company_name: Option<String>,           // 未入力: 固定のダミー値
    pub title: Option<String>,                  // 未入力: 固定のダミー値
    pub evaluation: Option<earnings::EarningsEvaluation>, // 未入力: Unrated
    pub embed_color: Option<String>,            // 未入力: グループ設定済みのembed_color("0x87EB87"形式)
    pub webhook_url: Option<String>,            // 未入力: グループ保存済みのwebhook_url。入力時は一時上書き(DB非保存)
    pub mention_targets: Option<Vec<String>>,   // 未入力: グループ保存済みのmention_targets。入力時は一時上書き(DB非保存)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestSendResponse {
    pub success: bool,
    pub failure_reason: Option<String>, // HTTP通信自体の失敗理由(タイムアウト・DNS等)、
                                         // または送信先が返した非2xxステータス・エラーメッセージ。
                                         // 「意図しない場所に届いた」等の到達先の正しさまでは検証しない(ユーザ責任範囲)
}

// ─── PUT /api/groups/bulk-destination ───
#[derive(Deserialize)]
pub struct BulkDestinationRequest {
    pub group_ids: Vec<GroupId>,
    pub config: GroupConfigDto, // 対象グループ全てに同じ設定を反映
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkDestinationResponse {
    pub updated_count: u32, // 一括インポート系(8章)と同様、処理件数を返す
}
```

### 対応関係

| 型 | 備考 |
|---|---|
| `GroupResponse` | `handlers/common.rs`配置、複数エンドポイントで共有 |
| `CreateGroupRequest` / `UpdateGroupRequest` | |
| `GroupConfigDto` | タグ付きUnion。`webhook_url`は復号済み平文で返す |
| `TestSendRequest` / `TestSendResponse` | 全フィールド任意、自由入力プレビュー送信、DB非保存。HTTPレベルの成否のみ判定。到達先の正しさはユーザ責任 |
| `BulkDestinationRequest` / `BulkDestinationResponse` | 同一設定を複数グループへ一括反映 |

### 残課題(次節以降で検討)

- `GroupConfigDto`のJSONタグ付けの実際の階層構造(`#[serde(tag = "medium", rename_all = "lowercase")]`とフィールドの`camelCase`変換の組み合わせ)は実装時に検証する
- `SlackConfig`側の`GroupConfigDto::Slack`は、`notifier::SlackConfig`同様、Discord実装完了後のMVP内拡張フェーズまで仮実装のまま

---

## 8. フィルタAPI(一括インポート含む)

### 経緯・方針

- **フィルタのenable/disable(`PATCH`)はリクエストボディ・レスポンスともに空(`ApiResponse::ok(())`)とする**。グループAPIの`pause`/`resume`とは異なり、一覧画面上での都度トグル操作である可能性が高く、全量を返す必要性は低いと判断
- **`created_groups`/`paused_groups`はグループ名の文字列配列ではなく、`GroupRef {id, name}`のペアで返す**。フロント側での後続操作(該当グループへの遷移等)の拡張性を考慮
- **インポート結果の警告(異常値検知、`EarningsWatch_Enabled`未入力等)は、個別フィールドに分けず`warnings: Vec<ImportWarning>`という汎用配列にまとめる**。項目が増えるたびに専用フィールドを増やす設計を避けるため
- **エクスポート(`/api/filters/export`、`/api/groups/{id}/filters/export`)は決算情報エクスポート(5章)と同じ`ExportFormat`(MVPでは`xlsx`のみ)を再利用する**(`handlers/common.rs`に配置、12章参照)

### 型定義

```rust
// ===== handlers/common.rs(追加) =====
use identity::{FilterId, GroupId};
use serde::Serialize;
use subscription::NotifyFilter;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterResponse {
    pub id: FilterId,
    pub group_id: GroupId,
    pub ticker: String,
    pub company_name: String,
    pub notes: Option<String>,
    pub enabled: bool,
}

impl From<NotifyFilter> for FilterResponse {
    fn from(f: NotifyFilter) -> Self {
        Self {
            id: f.id,
            group_id: f.group_id,
            ticker: f.ticker,
            company_name: f.company_name,
            notes: f.notes,
            enabled: f.enabled,
        }
    }
}

// グループ名解決用のペア(created_groups/paused_groupsで使用)
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRef {
    pub id: GroupId,
    pub name: String,
}
```

```rust
// ===== handlers/filter.rs =====
use identity::{FilterId, GroupId};
use serde::{Deserialize, Serialize};

// ─── GET /api/groups/{id}/filters ───
#[derive(Deserialize)]
pub struct ListFiltersQuery {
    pub page: u32,
    pub per_page: u32,
}
// レスポンスは Page<FilterResponse>

// ─── POST /api/groups/{id}/filters ───
#[derive(Deserialize)]
pub struct CreateFilterRequest {
    pub ticker: String,
    pub company_name: String,
    pub notes: Option<String>,
}
// レスポンスは FilterResponse

// ─── PUT /api/groups/{id}/filters/{filter_id} ───
#[derive(Deserialize)]
pub struct UpdateFilterRequest {
    pub ticker: String,
    pub company_name: String,
    pub notes: Option<String>,
}
// レスポンスは FilterResponse

// ─── PATCH .../enable, /disable, DELETE ... ───
// リクエストボディなし、レスポンスは ApiResponse::ok(())
// (グループのpause/resumeとは非対称。一覧上の都度トグル操作のため全量を返す必要性が低いと判断)

// ─── POST bulk-enable / bulk-disable / bulk-delete ───
#[derive(Deserialize)]
pub struct BulkFilterIdsRequest {
    pub filter_ids: Vec<FilterId>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkFilterActionResponse {
    pub updated_count: u32, // group.rsのBulkDestinationResponseと統一した命名
}

// ─── POST /api/filters/import(全体一括設定) ───
// フロントエンドがCSV/xlsxを共通JSON配列に変換済みのものを受け取る
#[derive(Deserialize)]
pub struct ImportFilterRow {
    pub ticker: String,             // EarningsWatch_Ticker
    pub company_name: String,       // EarningsWatch_CompanyName
    pub group_name: String,         // EarningsWatch_GroupName(全体用のみ)
    pub notes: Option<String>,      // EarningsWatch_Notes(任意)
    pub enabled: Option<bool>,      // EarningsWatch_Enabled(任意、未指定時はtrue扱い)
}

#[derive(Deserialize)]
pub struct ImportFiltersRequest {
    pub rows: Vec<ImportFilterRow>,
    // NOTE(本設計書での追加): ドライラン機能(03-features/import-export.md 9章)のために追加。
    // 型定義書時点では未反映だった記載漏れを本設計書で補った。
    #[serde(default)]
    pub dry_run: bool, // true: DBへ反映せずプレビュー結果のみ返す
}

// ─── POST /api/groups/{id}/filters/import(グループ単位) ───
#[derive(Deserialize)]
pub struct ImportGroupFilterRow {
    pub ticker: String,
    pub company_name: String,
    // group_nameを持たない(URLの{id}で確定済み)
    pub notes: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Deserialize)]
pub struct ImportGroupFiltersRequest {
    pub rows: Vec<ImportGroupFilterRow>,
    // NOTE(本設計書での追加): ドライラン機能(03-features/import-export.md 9章)のために追加。
    // 型定義書時点では未反映だった記載漏れを本設計書で補った。
    #[serde(default)]
    pub dry_run: bool,
}

// ─── インポート結果レスポンス(両インポートAPI共通) ───
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFiltersResponse {
    pub imported_count: u32,
    pub skipped_empty_rows: u32,
    pub duplicate_count: u32,
    pub error_rows: Vec<ImportErrorRow>,
    pub created_groups: Vec<GroupRef>, // 全体一括設定でのみ値が入る(グループ単位では常に空配列)
    pub paused_groups: Vec<GroupRef>,  // 同上
    pub warnings: Vec<ImportWarning>,  // 異常値検知・EarningsWatch_Enabled未入力等の警告をまとめる
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportErrorRow {
    pub row_number: u32, // 元ファイルの何行目か(ヘッダを除く連番、フロントでの該当行ハイライト用)
    pub reason: String,  // 例: "TickerとCompanyNameのどちらかが欠落しています"
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportWarning {
    pub row_number: u32,
    pub message: String, // 例: "tickerが20文字を超えています"、"EarningsWatch_Enabledが未入力のためtrueとして扱いました"
}

// ─── GET /api/filters/export, /api/groups/{id}/filters/export ───
#[derive(Deserialize)]
pub struct ExportFiltersQuery {
    pub format: crate::handlers::common::ExportFormat, // 決算情報エクスポート(5章)と同じenumを再利用
}
// レスポンス本体はxlsxバイナリ
```

### 対応関係

| 型 | 備考 |
|---|---|
| `FilterResponse` / `GroupRef` | `handlers/common.rs`配置 |
| `ListFiltersQuery` / `CreateFilterRequest` / `UpdateFilterRequest` | |
| `BulkFilterIdsRequest` / `BulkFilterActionResponse` | 一括有効化/無効化/削除で共通利用 |
| `ImportFilterRow` / `ImportFiltersRequest` | 全体一括設定用、`group_name`を持つ |
| `ImportGroupFilterRow` / `ImportGroupFiltersRequest` | グループ単位用、`group_name`を持たない |
| `ImportFiltersResponse` / `ImportErrorRow` / `ImportWarning` | 両インポートAPI共通。警告は汎用配列にまとめる |
| `ExportFiltersQuery` | `ExportFormat`(`common.rs`、12章)を再利用 |

### 残課題(次節以降で検討)

- xlsx生成処理(インポートと対になるエクスポート処理)の列構成定義の共通化方法

---

## 9. ユーザ設定API

### 経緯・方針

- **`user_settings`(`memo`のみを持つシンプルなテーブル)のドメイン型`UserSettings`は`subscription`クレートに配置する**(`02-types/subscription.md`)。「通知購読」という概念そのものとは無関係だが、新規クレートを増やさない方針のもと、`user_id`をキーに持つ「ユーザに紐づく設定値」という性質が`subscription`の既存の型(`NotifyGroup.user_id`等)と近いと判断した
- **パスワードの強度チェックは型に持たせず、`auth`クレート側のバリデーション関数(YaakoDriveの`password.rs`相当)に委ねる**
- **ユーザ名の重複時は`ApiErrorCode::AlreadyExists`(409)を使う**。メッセージで「そのユーザ名は既に使われています」等を伝える

### 型定義

```rust
// ===== subscription crate側 =====
use chrono::{DateTime, Utc};
use identity::UserId;
use serde::{Deserialize, Serialize};

// DB: user_settings(01-db-schema.md 4章)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub user_id: UserId,
    pub memo: Option<String>,
    pub updated_at: DateTime<Utc>,
}
```

```rust
// ===== api crate側(handlers/user_settings.rs) =====
use serde::{Deserialize, Serialize};
use subscription::UserSettings;

// ─── GET /api/users/me/settings ───
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettingsResponse {
    pub memo: Option<String>,
}

impl From<UserSettings> for UserSettingsResponse {
    fn from(s: UserSettings) -> Self {
        Self { memo: s.memo }
    }
}

// ─── PUT /api/users/me/settings ───
#[derive(Deserialize)]
pub struct UpdateUserSettingsRequest {
    pub memo: Option<String>,
}
// レスポンスは UserSettingsResponse

// ─── PUT /api/users/me/password ───
#[derive(Deserialize)]
pub struct UpdatePasswordRequest {
    pub current_password: String,
    pub new_password: String, // 強度チェックはauthクレート側のバリデーション関数(password.rs相当)に委ねる
}
// リクエスト成功時のレスポンスは ApiResponse::ok(())

// ─── PUT /api/users/me/username ───
#[derive(Deserialize)]
pub struct UpdateUsernameRequest {
    pub new_username: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUsernameResponse {
    pub username: String, // フロント側で表示中のユーザ名をその場で更新できるよう、変更後の値を返す
}
```

### 対応関係

| 型 | 置き場所 | 備考 |
|---|---|---|
| `UserSettings` | `subscription`クレート | |
| `UserSettingsResponse` / `UpdateUserSettingsRequest` | `api`クレート(`handlers/user_settings.rs`) | |
| `UpdatePasswordRequest` | `api`クレート(`handlers/user_settings.rs`) | 強度チェックは`auth`クレートのバリデーション関数に委ねる |
| `UpdateUsernameRequest` / `UpdateUsernameResponse` | `api`クレート(`handlers/user_settings.rs`) | 重複時は`AlreadyExists`(409) |

### 残課題(次節以降で検討)

- `auth`クレート側のパスワード強度バリデーション関数の具体的なシグネチャ・ルール(最低文字数等)は未確定

---

## 10. ダッシュボードAPI(ユーザ単位)

### 経緯・方針

- **「直近送信」「直近の送信失敗」は、件数集計ではなく実データ1件分の詳細(`NotifyHistoryResponse`)を返す**。管理者ダッシュボード(4章)と同様、実データ(グループ名・送信時刻等)を見たい場面が多いと判断
- これに伴い**`NotifyHistoryResponse`を`handlers/notify_history.rs`から`handlers/common.rs`へ移動**している(6章参照)。`dashboard.rs`からも参照するため
- **「送信媒体別数」は`NotifyMedium`が現状2種(Discord/Slack)で固定的なため、`HashMap`ではなく固定フィールドの構造体(`MediumBreakdown`)にする**。型安全性を優先

### 型定義

```rust
// ===== handlers/dashboard.rs =====
use crate::handlers::common::NotifyHistoryResponse;
use serde::Serialize;

// ─── GET /api/dashboard ───
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardResponse {
    pub group_count: u32,
    pub filter_count: u32,
    pub medium_breakdown: MediumBreakdown, // 送信媒体別グループ数
    pub paused_group_count: u32,           // 一時停止中グループ数
    pub webhook_missing_count: u32,        // webhook_url未設定のグループ数
    pub recent_sent: Option<NotifyHistoryResponse>,   // 直近送信(該当なしならNone)
    pub recent_failed: Option<NotifyHistoryResponse>, // 直近の送信失敗(status=Failedの直近1件、該当なしならNone)
}

// 送信媒体別数。NotifyMediumが現状2種で固定的なため、型安全な固定フィールド構造体にする
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediumBreakdown {
    pub discord: u32,
    pub slack: u32,
}
```

### 対応関係

| 型 | 備考 |
|---|---|
| `DashboardResponse` | `recent_sent`/`recent_failed`は`NotifyHistoryResponse`(`common.rs`)を再利用 |
| `MediumBreakdown` | `NotifyMedium`固定2種に対応した固定フィールド構造体 |

### 残課題(次節以降で検討)

- 集計クエリ(グループ数・フィルタ数・webhook未設定数等)の具体的なSQL実装は実装フェーズで検討する

---

## 11. お知らせ板・固定ページAPI

### 経緯・方針

- **お知らせ板・固定ページのドメイン型は、新規`content`クレートとして切り出す**(詳細は`02-types/content.md`)。決算・通知いずれとも性質が異なる「コンテンツ管理」であるため、既存クレートへの相乗りは避けた
- **`type`(blog/static)によってフィールドの意味が変わる(`display_order`はstatic限定)構造は、判別Union(`PageKind`)で表現する**。これまでのドメイン型は基本的にフラットな構造体+enumで表現してきたが、今回は`MentionTarget`と同様パターンとして判別Unionを採用する
- **一覧(`GET /api/pages`)は本文(`content_markdown`)を含まない軽量レスポンス、個別取得(`GET /api/pages/{id}`)のみ本文を含む**設計とする
- **投稿者名は`users.username`をそのまま返す**。個人・家族・友人向けの規模感を踏まえ、別途「表示名」の概念は追加しない
- **`UpdatePageRequest`(`PUT`)は`type`・`display_order`を含まない**。`type`はページ作成後に変更しない、`display_order`は専用API(`PATCH .../order`)で変更する設計とした。「既存の`static`ページを削除したい」というニーズは既存の`DELETE /api/pages/{id}`で対応済みのため、追加のAPIは不要
- **`pages.type`は専用enum型(`page_type`)としてDDLに追加している**(`01-db-schema.md` 10章、これまでの`EarningsEvaluation`等と一貫性を保つ)

### 型定義

```rust
// ===== content crate側(新設、詳細は02-types/content.md) =====
use identity::{PageId, UserId};

// (型定義は02-types/content.md参照)
```

```rust
// ===== api crate側(handlers/page.rs) =====
use chrono::{DateTime, Utc};
use content::PageType;
use identity::PageId;
use serde::{Deserialize, Serialize};

// ─── GET /api/pages?type= ───
#[derive(Deserialize)]
pub struct ListPagesQuery {
    pub r#type: PageType, // typeはRust予約語のためr#typeとする
    pub page: Option<u32>,     // blogのみページング対応。staticは全件返す想定
    pub per_page: Option<u32>,
}

// 一覧は軽量レスポンス(本文content_markdownを含まない)
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageListItemResponse {
    pub id: PageId,
    pub title: String,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub display_order: Option<i32>, // staticのみ値が入る(一覧のソート用にkindを分解して見せる)
    pub author_username: String,    // 投稿者名(usernameをそのまま使用)
}

// ─── GET /api/pages/{id} ───
// 個別取得は本文含む詳細レスポンス
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageDetailResponse {
    pub id: PageId,
    pub r#type: PageType,
    pub title: String,
    pub content_markdown: String,
    pub display_order: Option<i32>,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author_username: String,
}

// ─── POST /api/pages ───
#[derive(Deserialize)]
pub struct CreatePageRequest {
    pub r#type: PageType,
    pub title: String,
    pub content_markdown: String,
    pub display_order: Option<i32>, // type=staticの場合のみ必須(バリデーションは実装時)
    pub is_published: bool,
}
// レスポンスは PageDetailResponse

// ─── PUT /api/pages/{id} ───
#[derive(Deserialize)]
pub struct UpdatePageRequest {
    pub title: String,
    pub content_markdown: String,
    pub is_published: bool,
    // typeとdisplay_orderは含まない(typeは作成後変更不可、display_orderは専用API/order経由で変更する方針)
}
// レスポンスは PageDetailResponse

// ─── DELETE /api/pages/{id} ───
// リクエストボディなし、レスポンスは ApiResponse::ok(())

// ─── PATCH /api/pages/{id}/order ───
#[derive(Deserialize)]
pub struct UpdatePageOrderRequest {
    pub display_order: i32,
}
// レスポンスは PageDetailResponse、またはApiResponse::ok(())
```

### 対応関係

| 型 | 備考 |
|---|---|
| `ListPagesQuery` / `PageListItemResponse` | 本文を含まない軽量レスポンス |
| `PageDetailResponse` | 本文含む |
| `CreatePageRequest` / `UpdatePageRequest` / `UpdatePageOrderRequest` | 権限判定は既存の管理者権限(`Role::Admin`)を流用 |

### 残課題(次節以降で検討)

- `CreatePageRequest`で`type=static`時に`display_order`が必須になるバリデーションの実装方法(型レベルでは強制していない)
- ページ作成・編集・削除の管理者権限チェックの実装(`extractor.rs`等での`Role::Admin`判定)

---

## 12. handlers/common.rs 収録型 一覧(正)

検討の過程で複数の節にまたがって「`common.rs`に置く」と決めた型が分散していたため、ここに一覧としてまとめる。実装時はこの一覧を正とする。

| 型 | 由来する節 | 概要 |
|---|---|---|
| `GroupResponse` | 7章(グループAPI) | `NotifyGroup`のレスポンス変換 |
| `FilterResponse` | 8章(フィルタAPI) | `NotifyFilter`のレスポンス変換 |
| `GroupRef` | 8章(フィルタAPI) | `{id, name}`ペア。インポート結果の`created_groups`/`paused_groups`で使用 |
| `NotifyHistoryResponse` | 6章→10章で移動 | `notify_history`のレスポンス変換。`notify_history.rs`・`dashboard.rs`の双方から参照。`group_id`/`group_name`は`Option`(本設計書での変更点) |
| `ExportFormat` | 5章・8章で共有 | MVPでは`xlsx`のみ対応。決算情報・フィルタ両方のエクスポートAPIで共有する |

上記5型は`handlers/common.rs`に集約する。`ApiResponse<T>`/`ApiError`/`ApiErrorCode`/`Page<T>`(2章)は`response.rs`側であり、`common.rs`とは別ファイルである点に注意。

```rust
// ===== handlers/common.rs(ExportFormat定義) =====
use serde::Deserialize;

// MVPでは`xlsx`のみ対応。決算情報(5章)・フィルタ(8章)両方のエクスポートAPIで共有する
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Xlsx,
    // Csv, // 将来拡張(05-future-work.md)で追加
}
```

## 13. 命名・パターンの一貫性チェック

- **DTO命名規則**: 全ての`api`クレート内レスポンス型に`#[serde(rename_all = "camelCase")]`が付与されているか確認 → 3章の`LoginResponse`/`MeResponse`のみ単一フィールド(`username`)のため実質的に影響しないが、念のため他の型と統一する場合は付与しても害はない。他の章(4〜11章)は付与済み
- **判別Union(タグ付きenum)の使用箇所**: `MentionTarget`(`02-types/notifier.md`)、`GroupConfigDto`(7章)、`PageKind`(`02-types/content.md`)の3箇所。パターンとして統一されている
- **`From<Domain> for Response`変換の有無**: `EarningsResponse`(5章)、`NotifyQueueResponse`(6章)、`GroupResponse`(7章)、`FilterResponse`(8章)、`UserSettingsResponse`(9章)には`From`実装を用意しているが、`PageListItemResponse`/`PageDetailResponse`(11章)、`AdminUserResponse`/`CreateUserResponse`(4章)には用意していない。後者はJOIN結果(`author_username`等)を含むため単純な`From`変換にならず、意図的な非対称であるが、実装時に注記があった方が親切
