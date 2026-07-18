# EarningsWatch 本設計書 00. 概要・アーキテクチャ

> 本書は`仮設計書.md`(ver1)・`仮設計書からの追加点と変更点.md`・`仮設計書-型定義.md`の内容を統合した本設計書の一部である。
> 3ファイルの記述が食い違う場合は、後から書かれた`仮設計書からの追加点と変更点.md` → `仮設計書-型定義.md`の内容を優先して統合している。
> 本設計書は章ごとに複数のMarkdownファイルへ分割している。全体構成は以下の通り。

```text
design/
├── 00-overview.md       (本ファイル。概要・アーキテクチャ・依存関係)
├── 01-db-schema.md      (DDL全体)
├── 02-types/
│   ├── identity.md
│   ├── crypto.md
│   ├── earnings.md
│   ├── subscription.md
│   ├── notifier.md
│   ├── content.md
│   └── api.md
├── 03-features/         (機能ごとの業務ロジック・追加仕様)
├── 04-security.md        (認証・暗号化)
└── 05-future-work.md     (将来拡張の一覧)
```

## 目次
1. [目的とコンセプト](#1-目的とコンセプト)
2. [本書のスコープ(フロントエンドは対象外)](#2-本書のスコープフロントエンドは対象外)
3. [スコープ整理(MVP機能範囲 / MVP後拡張)](#3-スコープ整理mvp機能範囲--mvp後拡張)
4. [設計原則](#4-設計原則)
5. [技術スタック](#5-技術スタック)
6. [全体アーキテクチャ](#6-全体アーキテクチャ)
7. [開発フェーズ計画](#7-開発フェーズ計画)
8. [仮設計書からのTODO解消状況一覧](#8-仮設計書からのtodo解消状況一覧)

---

## 1. 目的とコンセプト

EarningsWatchは、個人・友人・家族向けに、上場企業等の決算速報をDiscord/Slackなどへ送信・通知するシステムである。

- 監視処理は単発実行型であり、定期実行そのものは cron / supercronic など外部スケジューラに委ねる。アプリ自身はスケジューラを持たない。
- 「監視」という言葉ではあるが、常時監視ではなく、まとめ/フィルターに近い。1日2回程度の定期実行を想定する。
- 各ユーザが個人ごとに設定を持てるようにし、送信内容がユーザ・グループ単位で分かれるようにする。
- インターフェース(Trait)を用いた抽象化を大切にし、通知媒体・スクレイピング対象サイトの追加/変更が容易にできる構成とする。これは実用性だけでなく、設計練習としての側面も持つ。

## 2. 本書のスコープ(フロントエンドは対象外)

**本書はバックエンドの設計に限定する。** フロントエンドの実装・設計は、バックエンドが完成した後に別途行う方針とする。

- API設計(`02-types/api.md`)のDTOは、Reactフロントエンドとの連携を前提に設計されている(フィールドは`snake_case`のまま定義し`#[serde(rename_all = "camelCase")]`でシリアライズ時のみ`camelCase`へ変換する、`02-types/`の各節を参照)。DTOの形自体はフロント連携ありきで確定しているが、実際の画面遷移・コンポーネント設計・状態管理などは本書の対象外である。
- Phase 0〜13(バックエンド9.9割完成)の間、動作確認用の最小限のReact製汎用フォーム(ログイン画面+任意API送信フォーム)のみを用意する(7章参照)。これは本格フロントエンド実装(Phase 14)の対象ではなく、疎通確認のためだけの簡易実装である。

## 3. スコープ整理(MVP機能範囲 / MVP後拡張)

### MVP機能範囲(Phase 0〜12、debug.pyによるダミースクレイパー使用)
- ログイン/ログアウト、Cookieベース認証(JWT + Refreshローテーション)
- 管理者による仮ユーザ作成/無効化(BAN)
- 通知グループのCRUD、一時停止/再開
- 通知フィルタ(証券コード・銘柄名)のCRUD、一時停止/再開、一括有効化/無効化/削除
- フィルタ一括インポート(CSV/Excel、全体/グループ単位、ドライラン対応)、フィルタエクスポート(xlsx)
- 送信先一括設定(グループ横断)
- Discord向け通知設定(webhook、Embed、メンション)。Slackは`notify_slack_configs`を仮カラムのまま用意し、Discord実装完了後にMVP内拡張フェーズで詳細仕様を確定する
- テスト送信(test-send、送信内容の自由入力プレビュー拡張含む)
- monitor(スクレイピング)/notify(送信)のCLI実行、fingerprintによる新規判定パイプライン(スクレイピング対象はdebug.pyのダミー値)
- ロギング(SqlLayer/MemoryLayerによるDB保存・管理者通知)
- ユーザダッシュボード、管理者ダッシュボード
- 決算情報ログ画面(xlsx出力、フィルタ)
- お知らせ板・固定ページ(管理者マークダウン投稿、全ユーザ閲覧)

### MVP後(バックエンド9.9割完成後に着手する範囲)
- Phase 13: 実サイトへのスクレイピング実装(Playwright、debug.pyの置き換え)
- Phase 14: フロントエンド本格実装

### さらにMVP後に回すもの(将来拡張、`05-future-work.md`で詳述)
- 送信フォーマットの複数指定、Email送信媒体、HTTPS化、スマホ対応レスポンシブUI、カーソル型ページングへの切替、Slack実装(TODO#1として先送りではなくMVP内拡張だが、Slack自体の詳細仕様確定までは将来的な調査項目として残る)、webhook暗号鍵ローテーション、決算評価別Embed色自動出し分け、お知らせ板未読バッジ、管理者ごとの個別通知先設定 等

## 4. 設計原則

1. **依存関係逆転を徹底する**:通知媒体・スクレイピング対象サイト・DB実装はいずれもTraitの背後に隠し、具象クレート(`notifier`/`scraper`/`infra`)を差し替え可能にする。
2. **`shared`クレートを作らない**:共通型は責務ごとのクレートに個別定義し、エラーは伝搬で対処する。
3. **状態の正規化ロジックは1箇所に集約する**:fingerprint生成、ticker正規化はいずれも`earnings`クレート内の単一関数に集約し、表記揺れによる二重登録を防止する。
4. **冪等性を意識する**:`migration`は差分適用のみ行う。CSV/Excel一括インポートは「一新」ではなく差分検出・反映とする。
5. **Rust/Pythonの責務を明確に分離する**:Pythonは生データ取得のみを担い、状態・ループ・判定ロジックは一切持たない。ロジックはすべてRust側に置く。
6. **失敗時の可観測性を優先する**:送信失敗・スクレイピング失敗はエラーコードとメッセージで具体的に返し、原因追跡ができるようにする。
7. **個人利用規模であることを前提にしすぎない**:Tailscale閉域を前提としつつも、将来の一般公開に耐えられる認証・暗号化設計を選ぶ(YaakoDriveと同じ方針)。

## 5. 技術スタック

| 分類 | 採用technology |
|---|---|
| フロントエンド | React(本書スコープ外。2章参照) |
| バックエンド | Rust(Cargo Workspace) |
| HTTP API | axum |
| スクレイピング | Python(Playwright) |
| DB | PostgreSQL |
| DBアクセス | sqlx |
| コンテナ | Docker / Docker Compose |
| 認証 | JWT + HttpOnly Cookie + Refresh Tokenローテーション |
| パスワードハッシュ | argon2 |

補足:スクレイピングはRustでは行わずPythonを使用する。Playwrightの方がより自然にページを開けるため。

### 主要crate候補(Rust側)
```toml
# HTTP / async
axum
tokio
tower
tower-http

# DB
sqlx

# serialize
serde
serde_json

# error
thiserror

# log / tracing
tracing
tracing-subscriber

# auth
jsonwebtoken
argon2
rand

# time / id / hash
chrono
uuid
sha2      # fingerprintのハッシュ化に使用

# async trait / process
async-trait
tokio::process   # Pythonプロセス起動

# config
config
dotenvy

# cookie
axum-extra

# CLI
clap

# 暗号化(webhook_url等)
aes-gcm

# test
tempfile
```

## 6. 全体アーキテクチャ

### 6.1 エントリポイント
`server`と`cli`の2エントリポイントとする。

- **server**:フロントエンドでの変更などを反映する常駐バイナリ。web処理を担当。
- **cli**:監視処理実行・送信処理実行・ユーザ作成など、mainが呼び出されるだけの単発実行。`clap`のサブコマンドで処理を分岐する。

想定コマンド:
```text
create-admin: 管理者ユーザを作成する
migration:    SQLのマイグレーション。未適用のマイグレーションファイルのみ差分適用する
monitor:      監視処理(スクレイピング処理)をする
notify:       送信処理をする
```

スクレイピング(実行の30分前など)と送信を別コマンド・別タイミングで実行することで、送信時間が実行時間によってブレないようにする。

`migration`はsqlxの管理テーブルにより「どこまで適用済みか」を記録するため、何度実行しても未適用分のみが適用される。この性質を利用し、開発中と本番でそれぞれ以下のタイミングで実行する。

- 開発中:スキーマファイルを変更したタイミングで、手動で`cli migration`を実行
- 本番:コンテナ起動時に必ず実行(`entrypoint.sh`)

### 6.2 Workspace構成
DDD・クリーンアーキテクチャを意識し、依存関係逆転によって通知媒体・スクレイピング対象サイト・DBの実装を差し替えやすくする。`shared`クレートは作らず、構造体は各責務のクレートに配置し、エラーは伝搬で対処する。

型定義作業を通じて、当初の仮設計書には無かった`crypto`・`content`の2クレートを新設した(6.4節参照)。

```text
backend/
├── Cargo.toml
└── crates/
    ├── server        # HTTPサーバ起動、DI組み立て
    ├── cli           # 管理コマンド、monitor/notify実行
    ├── app           # ユースケース層
    ├── api           # axum HTTP層
    ├── auth          # 認証/JWT/PasswordHasher
    ├── identity      # UserId, GroupId, FilterId, PageId, RefreshTokenIdなどのID型
    ├── crypto        # (新設)webhook_url等の機密情報の汎用暗号化型(Encrypted<T>/Plain<T>)
    ├── earnings      # 決算情報のコアドメイン(Earnings, EarningsEvaluation, fingerprint正規化)
    ├── subscription  # 通知フィルタ・グループの業務ルール、送信管理(NotifyQueueEntry等)、UserSettings、SystemNotifyConfig
    ├── notifier      # 通知媒体 Trait + 媒体ごとの実装(discord/slack)
    ├── scraper       # スクレイピング対象 Trait + サイトごとの実装(Python連携、stdout経由のJSON受け渡し)
    ├── content       # (新設)お知らせ板・固定ページ(pages)のドメイン型
    ├── repository    # Repository Trait / UnitOfWork Trait
    ├── infra         # PostgreSQL/sqlx実装
    ├── logging       # tracing Layer(SqlLayer/MemoryLayer)、LogLevel/LogProcess/LogEntry
    └── config        # 設定読み込み
```

### 6.3 各クレートの役割
| クレート | 役割 |
| --- | --- |
| `server` | HTTPサーバの起動とDI組み立て。`infra`の具象Repository、`notifier`/`scraper`の具象実装、`app`のユースケース、`api`のRouterを組み合わせる |
| `cli` | 管理コマンド(create-admin, migration)、monitor/notify実行。HTTPを経由せず単発実行される |
| `app` | ユースケース層。ログイン、フィルタ管理、CSVインポート、monitor/notifyの実行フローなどを実装。HTTP・axum・SQL・PostgreSQLには依存しない |
| `api` | HTTP層。Routing、Request解析、Response生成、Cookie処理、JWT認証ミドルウェア、エンベロープ形式へのエラー変換を担当 |
| `auth` | JWT生成・検証、PasswordHasher、TokenClaims、Role等 |
| `identity` | `UserId`、`GroupId`、`FilterId`、`PageId`、`RefreshTokenId`などのID型 |
| `crypto`(新設) | 機密情報の汎用暗号化型`Encrypted<T>`/`Plain<T>`(用途タグによる型レベル区別)。AES-256-GCMによる暗号化・復号ロジック |
| `earnings` | 決算情報のコアドメイン。`Earnings`、`EarningsEvaluation`、fingerprintの生成・正規化(ticker正規化含む)ロジックを1関数に集約して持つ |
| `subscription` | 「誰が何を監視しているか」の業務ルール。`NotifyGroup`、`NotifyFilter`、一時停止/再開ルールに加え、送信管理(`NotifyStatus`/`NotifyQueueEntry`/`NotifyHistoryEntry`)、`UserSettings`、`SystemNotifyConfig`(管理者共有の通知先設定)を担当 |
| `notifier` | 通知媒体のTraitと媒体ごとの実装。`discord.rs`/`slack.rs`のようにモジュールを追加するだけで媒体を追加できる |
| `scraper` | スクレイピング対象のTraitとサイトごとの実装。Pythonプロセスを起動し、`stdout`経由でJSONを受け取る |
| `content`(新設) | お知らせ板・固定ページ(`pages`)のドメイン型。`PageKind`による判別Union(`blog`/`static`の性質差を表現) |
| `repository` | Repository Trait / UnitOfWork Trait。SQLやsqlxには依存しない |
| `infra` | Repository/UnitOfWork TraitのPostgreSQL/sqlx実装。SQL・sqlx・PostgreSQL固有処理はここに閉じ込める |
| `logging` | tracingのLayer実装(`SqlLayer`/`MemoryLayer`)、`LogLevel`/`LogProcess`/`LogEntry`ドメイン型 |
| `config` | 設定ファイル・環境変数の読み込み |

### 6.4 依存関係

型定義作業中に発覚した追加分(`crypto`/`content`の新設に伴うもの、`subscription -> earnings`)を反映した最終形は以下の通り。

```text
server       -> api, app, infra, config, logging
cli          -> app, infra, config, logging, scraper, notifier
api          -> app, auth, identity, content
app          -> auth, earnings, subscription, notifier, scraper, repository
infra        -> repository, auth, earnings, subscription, identity
repository   -> auth, earnings, subscription, identity
subscription -> identity, crypto, earnings
earnings     -> identity
notifier     -> identity, crypto
scraper      -> earnings
content      -> identity
auth         -> identity
crypto       -> 外部crateのみ
config, identity, logging -> 外部crateのみ
```

追加された依存とその理由:

| 追加された依存 | 理由 |
|---|---|
| `crypto -> 外部crateのみ`(新規クレート) | webhook_url等の暗号化型を`notifier`固有から汎用化 |
| `notifier -> crypto` | `Encrypted<WebhookUrlTag>`の利用 |
| `subscription -> crypto` | `SystemNotifyConfig`の`webhook_url`暗号化(`SystemNotifyWebhookUrlTag`) |
| `subscription -> earnings` | `NotifyQueueEntry`/`NotifyHistoryEntry`が`earnings`由来の型(`EarningsEvaluation`等)や非正規化列を持つ |
| `content -> identity`(新規クレート) | `PageId`の利用 |
| `api -> content` | `handlers/page.rs`が`content::PageKind`等を参照 |

`scraper`が`earnings`に依存するのは、スクレイピングで取得した生データからfingerprintを組み立てる際、`earnings`クレートが持つ正規化ロジック(ticker正規化含む)を利用するため。

### 6.5 エラー方針
各クレートに専用のError型/Result型を置き、`app`クレートで各クレートのエラーをまとめる`AppError`/`AppResult`を定義する。`shared`クレートは作らず、エラーは伝搬で対処する。

## 7. 開発フェーズ計画

```text
Phase 0:  リポジトリ土台
Phase 1:  Rust workspace土台(Cargo Workspace、crates構成)
Phase 2:  config / logging(SqlLayer/MemoryLayer) / health check
Phase 3:  DB / sqlx migration(全テーブル一括作成)
Phase 4:  domain型 / repository trait
Phase 5:  infra実装(sqlx実装)
Phase 6:  CLI管理コマンド(create-admin, migration)
Phase 7:  認証API(login/refresh/logout/me、JWT+Cookie)
Phase 8:  グループ/フィルタAPI(CRUD、一時停止/再開、一括操作)
Phase 9:  CSV/Excelインポート・エクスポートAPI
Phase 10: ダッシュボード/送信履歴/管理者API/お知らせ板API
Phase 11: debug.pyを使ったmonitor/notify CLI実装
Phase 12: 統合テスト/運用確認
Phase 13: 実スクレイピング実装(Python/Playwright、debug.pyを本物に置き換え)。この間、動作確認用の簡易React汎用フォーム(2章参照)を用意する
Phase 14: フロントエンド本格実装へ移行
```

### 7.1 Phase 0の内容
各種プロジェクト・周辺ファイルを揃え、コードはまだ書かずに「開発を始められる状態」にする。

- フロントエンドプロジェクト作成(簡易汎用フォーム用途を兼ねる、2章参照)
- バックエンドプロジェクト作成(Cargo Workspace土台)
- sqlファイルの配置場所フォルダ作成
- configファイル配置場所フォルダ作成
- ダミースクレイパー(debug.py)作成
- Docker系一式(Dockerfile、compose.yaml/compose.dev.yaml/compose.prod.yaml)
- `.gitignore`、`.dockerignore`
- 実行を容易化するMakefile群の整備

`entrypoint.sh`はこの時点では`cli`コマンドが未実装のため、`cli migration`部分をコメントアウトしておく(Phase 6のCLI実装完了後に有効化)。

### 7.2 debug.pyについて
最初に、固定値を返すダミースクレイパー(`debug.py`)を作成する。実際のスクレイピング処理(Playwrightでの実サイトアクセス)は不安定さ・サイト構造変更の影響を受けやすいため、これを後回しにし、まずは固定値を返すdebug.pyを使って「fingerprint判定→DB保存→notify送信」という一連のパイプラインを先に完成させる。実スクレイピングへの置き換えはPhase 13で行う。

### 7.3 「9.9割完成」の基準(Phase 12時点)
発見された軽微な問題はすべて修正済みで、MVPとして実用化できる段階を指す。未発見の細かな不具合は残っている可能性があるが、既知の課題は残さない状態とする。

### 7.4 フロントエンドは最後
バックエンド(Phase 0〜13)が9.9割完成してから、フロントエンド本格実装(Phase 14)に着手する方針とする。Phase 13終了〜Phase 14着手までの間の動作確認は、簡易React汎用フォーム(ログイン画面+任意APIパス・メソッド・JSONボディ手打ち送信フォーム)で行う。個別のドメイン別画面は作らず、CSV/Excelインポート等の確認もこのフォームからJSONを直接送信して代用する。Discord/Slackへの実送信結果は、Discord/Slack側の画面を直接見て確認する(フロントエンド側では対応しない)。

## 8. 仮設計書からのTODO解消状況一覧

仮設計書(ver1)19章に記載されていたTODO全11件は、以下の通りすべて解消済みである。

| # | 項目 | 解消内容の所在 |
|---|---|---|
| 1 | Slack通知設定の詳細仕様 | Discordを先行実装し、Slackの詳細はMVP内拡張フェーズで決定する方針として解消(`03-features/notification.md`) |
| 2 | CSV/Excel業務ルールの異常値検知閾値 | 文字数上限による検知方式・閾値を決定(`03-features/import-export.md`) |
| 3 | mention_targetsの記法変換ロジック | Discord向け`type:value`形式・`allowed_mentions`変換ロジックを決定(`03-features/notification.md`) |
| 4 | スクレイピング判別用フィールド構成 | fingerprint対象3項目(タイトル・書き出し・決算評価)を決定(`03-features/scraping.md`) |
| 5 | 時間ベース/件数ベースクエリの具体値の妥当性検証 | 件数ベース1本化・直近100件に設計変更(`03-features/scraping.md`) |
| 6 | 本番Cookie SecureフラグとTailscale運用の整合性 | 本番・開発ともにSecure=falseとし、HTTPS化後に本番のみtrueへ切替と決定(`04-security.md`) |
| 7 | webhook_url暗号鍵のローテーション運用 | MVPでは実装せず将来拡張へ(`05-future-work.md`) |
| 8 | Phase 13〜14間のフロントエンド仮UIの要否 | 簡易React汎用フォームを作成する方針で決定(本書2章・7.4節) |
| 9 | お知らせ板・パッチ通知の設計 | `pages`テーブル新設・`content`クレート新設により解消(`03-features/notice-board.md`) |
| 10 | フィルタデータのExcel出力(グループ別シート分け) | シート分けは行わずxlsx1シート出力と決定(`03-features/import-export.md`) |
| 11 | フィルタの一括有効化/無効化/削除 | 専用一括APIを新設して解消(`03-features/notification.md`または`02-types/api.md`) |

> TODO#9は仮設計書ver1本体(19章)では「未解決」表記のまま残っているが、これは本体を書き換えない運用方針によるものであり、本設計書では上表の通り解決済みとして扱う。
