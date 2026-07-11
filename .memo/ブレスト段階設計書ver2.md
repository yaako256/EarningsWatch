# EarningsWatch 設計書(ブレスト段階ver2)

## 目的
EarningsWatchは、個人・友人・家族向けの、上場企業等の決算速報をDiscordなどに送信・通知するシステムである。

- 監視処理は単発実行型であり、定期実行は cron / supercronic など外部スケジューラに委ねる
- 「監視」といっても、まとめ/フィルターのイメージに近く、1日に2回程度の定期送信を想定
- 各設定を個人でもてるようにし、送信内容が分かれるようにする
- インターフェースの概念を大切にし、通知対象やスクレイピング対象の変更等が容易にできるようにする練習も兼ねる

## 仕様技術
- フロントエンド: React
- バックエンド: Rust (Cargo Workspace)
- HTTP API: axum
- スクレイピング: Python (Playwright)
- DB: PostgreSQL
- DBアクセス: sqlx
- コンテナ: Docker / Docker Compose
- 認証: JWT + HttpOnly Cookie + Refresh Tokenローテーション
- パスワードハッシュ: argon2

補足: スクレイピングはRustでは行わずPythonを使用する。Playwrightの方がより自然にページを開けるため。

## 送信先(通知先)
現段階では以下を想定。
- Discord
- Slack
- Email

## エラー伝搬
前回のプロジェクト同様、shareクレートは作らない。共通型はそれぞれのクレートに個別定義し、エラーは伝搬で対処する。

## エントリポイント
`server`と`cli`の2エントリポイントとする。

### server
フロントエンドでの変更などを反映する常駐バイナリ。web処理を担当。

### cli
監視処理実行・送信処理実行・ユーザ作成など、mainが呼び出されるだけの単発実行。`clap`のサブコマンドで処理を分岐する。

想定コマンド:
```
create-admin: 管理者ユーザを作成する
migration:    SQLのマイグレーション。初期化時に実行する
monitor:      監視処理(スクレイピング処理)をする
notify:       送信処理をする
```

スクレイピング(30分前など)と送信を別コマンド・別タイミングで実行することで、送信時間が実行時間によってブレないようにする。

## スクレイピング設計(一覧→個別ページ問題)

サイトは「一覧→個別ページ」と遷移する必要があるが、決算件数が多いため個別ページへ毎回遷移するのはコストが高い。そのため、一覧ページの段階で新規/既知を判別し、個別ページへは新規分のみ遷移する。

### 差分検出方式: fingerprint
一覧ページで取得できる情報(証券コード・公開日時・決算タイトルなど)のみを使い、専用構造体からハッシュ化した値を`fingerprint`として新規/既知の判別キーとする。

- fingerprintの生成(結合・正規化・ハッシュ化)は**Rust側の1つの関数に集約**する(表記揺れによる重複判定防止のため)
- 個別ページ側ではfingerprintを再計算しない。一覧取得時点で確定する不変な識別子として扱う
- DBには決算情報の履歴と並行してfingerprint(ハッシュ化後文字列)を保存する

### Rust/Python間の責務分担
- Python: 一覧ページの生データ(判別用フィールド群、個別ページへのURL)をJSONで返すのみ
- Rust: 生データからfingerprintを組み立て・ハッシュ化し、DBの既知fingerprintと突き合わせて新規/既知を判別

これにより正規化ルールの二重管理を避ける。

### 差分検出クエリ(時間ベース/件数ベースの2段階)
決算集中期(3月・11月など)の件数急増を考慮し、既知fingerprintの取得範囲を2段階とする。

1. まず時間ベースで取得(例: 直近7日分)
2. 件数がn件に満たない場合、件数ベースで取得し直す(例: 直近n件)

件数ベースのみだと決算集中期に本来直近の対象が漏れるリスクがあるため、時間ベースを優先し、閑散期の安全マージンとして件数ベースを併用する(2回のクエリに分ける)。

### ページ送り制御
- 該当ページの全件が新規と判定されたら、次ページを取得する
- 既知のfingerprintが1件でも検出されたら、その時点でページ送りを打ち切る

制御ループはRust(`monitor`)側が持ち、Pythonは`list --page N`のようにページ番号を受け取り、該当ページの生データを返すだけとする(スクレイピング特有の状態やロジックをPython側に持たせない)。

### 処理フロー(まとめ)
```
1. Rust(monitor)がPythonに `list --page 1` を実行させる
2. Rustが返却された生データからfingerprintを組み立てる
3. DBから既知fingerprint(時間ベース→不足時は件数ベース)をHashSetとして取得
4. 一覧のfingerprintとHashSetを突き合わせ
   - 全件新規 → 次ページを取得(1に戻る、pageをインクリメント)
   - 既知が1件でもあれば → ページ送りを打ち切り、新規分を確定
5. 新規分のURLのみ、個別ページへ遷移して詳細情報を取得
6. 個別ページの内容と合わせて `MonitoredEarningsReport` として正規化する
```

### 構造体案(現時点)

#### 新規判定用(サイト固有・monitorクレート内のサイト専用モジュールでのみ持つ)
```rust
pub struct ItiranScraperOutput {
  pub items: Vec<EarningItems>,
}

pub struct EarningItems {
  // 未定。判別に使えるもの。この文字列をそのままfingerprintとする
  // 結合・正規化・ハッシュ化はRust側の1関数に集約する方針は決定済み
  pub payload: String,

  // 個別ページへのURL
  pub url: String,
}
```

#### 新規判別後の出力(サイト共通・notifyフローへ渡す最終正規化済み構造体・monitorクレートで持つ)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredEarningsReport {
  pub schema_version: u32,      // Jsonの仕様変更時にRust側でスキーマを分ける用
  pub source: String,           // どのサイトで取得したか
  pub fetched_at: DateTime<FixedOffset>, // スクレイピング開始時間
  pub items: Vec<Earnings>,     // 新規決算内容
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Earnings {
  pub ticker: String,
  pub company_name: String,
  pub published_at: DateTime<FixedOffset>,
  pub title: String,
  pub url: String,
  pub summary: String,
  pub evaluation: EarningsEvaluation,
}

pub enum EarningsEvaluation {
    Positive,
    Neutral,
    Negative,
    Unrated,
}
```

#### DB保存用(サイト共通・構造体としては宣言しない)
`MonitoredEarningsReport`の`items`を以下のカラム構成でDBに保存する。

- `ticker`: 証券コード
- `company_name`: 会社名
- `published_at`: 公開時間
- `title`: 決算のタイトル
- `url`: 詳細情報へのURL
- `evaluation`: 決算評価
- `fingerprint`: 新規判別用fingerprint(Rustでハッシュ化)
- `source`: 取得元サイトのラベル

### 決算評価(`EarningsEvaluation`)の判定ロジック
Rust側で独自の判定ロジックは持たない。スクレイピング元サイトが既に表示している評価をそのまま正規化して転記する。将来評価情報を持たないサイトへ切り替わった場合は`Unrated`として扱う(既存enumで対応可能)。

## ロギングとその送信
`tracing`クレートを用い、ログの発生箇所(呼び出し側)と処理方法(保存・通知)をLayerとして分離する。呼び出し側は`info!`/`warn!`/`error!`などのマクロでイベントを発行するだけでよい。

### Layer構成
- `SqlLayer`: ログをSQLに保存(全体ロガー相当)
- `MemoryLayer`: warn/error以上をメモリに貯め、管理者が設定した通知先(Discordなど)へ送信するトリガーを持つ(定期実行時ロガー相当)

登録するLayerはエントリポイントによって異なる。
- `server`起動時: `SqlLayer`のみ登録
- `cli monitor` / `cli notify`起動時: `SqlLayer` + `MemoryLayer`を登録

### SQL保存(`SqlLayer`)の書き込み方式
`on_event`は同期関数のため直接`.await`できない。ログ発生頻度が高くなりうるため、都度同期的にDB書き込みへ行くのはレイテンシ・負荷両面で不利。よって非同期バッチ書き込み方式とする(YaakoDNSのクエリログと同様の構成)。

- `on_event`はログエントリをチャネルに送るのみ(非ブロッキング)
- 別タスクがチャネルを受信し、メモリバッファに蓄積した上でPostgreSQLへバルクINSERT

flush条件:
- n件溜まったら
- (`server`のみ)フロントエンドからログ表示のリクエストが来たら
- (`cli monitor` / `cli notify`のみ)単発実行が終わったら(最終flush)

`server`と`cli`は別プロセスのため、バッファ・flushタスクもプロセスごとに独立して存在する。

### ログエントリに含める情報と取得方法
| 情報 | 取得方法 |
|---|---|
| ログレベル | `event.metadata().level()`から自動取得 |
| 発生箇所(ファイル/行/モジュール) | `event.metadata()`から自動取得 |
| 発生時刻 | 自動付与されないためLayer側で`Utc::now()`等を取得しセット |
| メッセージ・任意のフィールド内容 | `Visit`トレイトを実装し`event.record()`で走査して組み立て |
| どのプロセス由来か(server/monitor/notify) | Layer初期化時にプロセス種別を固定値として持たせ付与 |

「どのプロセス由来か」はDBのログテーブルにカラムとして明記する。

### 通知(`MemoryLayer`)の送信先
定期実行時(`cli monitor` / `cli notify`)にwarn/error以上が発生した場合、実行終了時にメモリバッファの内容を、管理者が設定した通知先(Discordなど)へ送信する。

`server`側で見逃せない重大エラーが発生した場合も同じ通知先へ送信する経路を用意する。`server`は常駐プロセスのため「実行終了時にflush」という`cli`側のトリガーは適用できず、`MemoryLayer`は責務を分けて設計する。

- **バッファリング責務**: warn/error以上をメモリに貯める(`server`/`cli`共通)
- **flushトリガー責務**: いつ通知先へ送るかはプロセスの性質によって異なるため外部から注入できるようにする
  - `cli`(monitor/notify): プロセス終了時にflush
  - `server`: error以上の発生時に即時、または短い時間窓でflush(詳細は実装時に検討)

## 通知フィルタ・グループ設定

### フィルタの枠組み
- ユーザごとに設定
- グループを作れる
- グループごとに送信先も異なる可能性あり
- 銘柄名の揺れも考慮し、証券コードと銘柄名の両方でフィルタ。片方一致で送信内容に含める

### フィルタ設定方法
- フロントエンドで設定
- 視覚的な設定方法とCSV/JSONなどコード的な設定方法を想定
- CSV/ExcelのDrag & Dropによる一括設定も想定(ユーザ全体向け・グループ単体向けの2種類)

### 送信先の指定
- グループ毎の送信先指定はフロントエンドの視覚的操作で設定可能
- 送信先の一括指定も可能にする
- Discordの場合は専用設定項目を用意(Embed使用有無、色など)

### 用語の整理(混乱防止)
「一括設定」という言葉がCSV由来とフロントエンド由来の2種類で使われ紛らわしいため、以下のように呼称を分ける。

- **フィルタ一括インポート**: CSV/ExcelのDrag & Dropによる、フィルタ内容(証券コード・銘柄名など)の一括登録・更新
- **送信先一括設定**: フロントエンドの視覚的操作による、複数グループへの送信先設定の一括反映

## CSV/Excelでのフィルタ内容一括設定
- Drag & Dropで設定する
- 列番号指定ではなく、ヘッダ行の値による設定項目の抽出とし、各ユーザが他の情報も含めた設定を可能にする
- 列名の重複を避けるため、アプリ名を列名に含める
- 空行はスキップ
- 壊れた行/足りない行があれば変更を反映せずエラーを出す
- その他列の内容は無視する
- 一新ではなく、バックエンドで変更部分を検出/反映する

### 全体での一括設定(ユーザ全体の設定を一新)
想定列名:
- `EarningsWatch_Ticker`: 証券コード
- `EarningsWatch_CompanyName`: 会社名
- `EarningsWatch_GroupName`: フィルタグループ
- `EarningsWatch_Notes`: 備考欄

### グループ毎の一括設定
想定列名:
- `EarningsWatch_Ticker`: 証券コード
- `EarningsWatch_CompanyName`: 会社名
- `EarningsWatch_Notes`: 備考欄

### `notify_filters`の重複行の扱い
同一グループ内で同じ`ticker`+`company_name`が重複登録されても、UNIQUE制約は設けずエラーは出すが登録自体は許可する。

- 動作に支障がないため
- ユーザが同じ銘柄に複数の備考(`notes`)を残したい場合があるため
- 後から変更が容易なため

フィルタ一括インポート時も同様の方針とし、重複行があってもインポート自体は継続する。インポート結果画面またはログで「n件重複の可能性あり」等の警告を表示する。

## ユーザ画面
想定画面は以下の2つ。

- フィルタ設定画面
  - 全体設定
  - グループ毎の設定(送信先/送信先ごとの設定など)
  - グループ毎の送信フィルタ
- ダッシュボード(何銘柄を監視してるかなど)

## JWT/Cookieのフロー
別プロジェクト(YaakoDrive)の設計をそのまま流用する。

- アクセストークン有効期限: 15分(config管理)
- リフレッシュトークン有効期限: 30日(config管理)
- リフレッシュタイミング: リクエストが401を受けたらリフレッシュを実行し、元のリクエストを再試行する(先読み更新はしない)

## スクレイピング対象サイトの利用規約
対象サイトにスクレイピング行為を禁止する規約がないことを確認済み(プライバシーポリシーより)。

## DBスキーマ

### ロギング
```sql
CREATE TYPE log_process AS ENUM (
    'server',
    'monitor',
    'notify'
);

CREATE TABLE logs (
  -- 自動採番ID
  id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  -- ログの発生時刻
  timestamp   TIMESTAMPTZ NOT NULL,
  -- ログレベル
  level       VARCHAR(5) NOT NULL
    -- ログ項目のチェック
    CHECK (level IN ('TRACE','DEBUG','INFO','WARN','ERROR')),
  -- どのプロセス由来か
  process     log_process NOT NULL,
  -- ログ発生場所
  target      TEXT NOT NULL,
  -- メッセージ
  message     TEXT,
  -- 構造化ログ用
  fields      JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX idx_logs_timestamp ON logs (timestamp DESC);
CREATE INDEX idx_logs_process ON logs (process);
```

### ユーザ
既に作成済みの別アプリ(YaakoDrive)のものを流用する。
```sql
CREATE TABLE users (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    username        TEXT        NOT NULL,
    password_hash   TEXT        NOT NULL,
    role            TEXT        NOT NULL DEFAULT 'user',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    disabled_at     TIMESTAMPTZ
);

CREATE UNIQUE INDEX users_username_unique ON users(username);
```

### リフレッシュトークン
既に作成済みの別アプリ(YaakoDrive)のものを流用する。
```sql
CREATE TABLE refresh_tokens (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT        NOT NULL,
    user_agent  TEXT,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at  TIMESTAMPTZ
);

CREATE UNIQUE INDEX refresh_tokens_token_hash_unique ON refresh_tokens(token_hash);
```

### ユーザごとの設定項目
サブタイプは、sqlxの型安全を利かせるため媒体ごとに専用テーブルを作る方針。処理上は証券コードと銘柄名の片方が埋まっていればいいが、見た目上どちらも必須とする。フロントエンドで送信媒体などの設定を切り替えても設定を残すため、グループ作成時に全媒体テーブルへ必ず1行作成し以後残し続ける。グループ一括設定の項目はDBに持たせず、フロントエンド側の機能として実現する。SlackなどはtoDoのため詳細は後で調べる。


```sql
CREATE TYPE notify_medium AS ENUM ('discord', 'slack', 'email');

-- ユーザ個人設定
CREATE TABLE user_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    memo TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- グループ本体
CREATE TABLE notify_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    medium notify_medium NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Discord固有設定(グループ作成時に必ず1行作成し、以後残し続ける)
CREATE TABLE notify_discord_configs (
    group_id UUID PRIMARY KEY REFERENCES notify_groups(id) ON DELETE CASCADE,
    webhook_url TEXT,  -- NULL許容。未設定時はDiscord送信を試みない
    embed_color TEXT,
    mention_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    mention_name TEXT
);

-- Slack固有設定(フィールド未定、仮カラム)(後で調べる)
CREATE TABLE notify_slack_configs (
    group_id UUID PRIMARY KEY REFERENCES notify_groups(id) ON DELETE CASCADE,
    todo_a TEXT,
    todo_b TEXT,
    todo_c TEXT
);

-- Email固有設定(フィールド未定、仮カラム)(後で調べる)
CREATE TABLE notify_email_configs (
    group_id UUID PRIMARY KEY REFERENCES notify_groups(id) ON DELETE CASCADE,
    todo_a TEXT,
    todo_b TEXT,
    todo_c TEXT
);

-- フィルタ設定(グループに対して複数)
CREATE TABLE notify_filters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES notify_groups(id) ON DELETE CASCADE,
    ticker TEXT NOT NULL,
    company_name TEXT NOT NULL,
    notes TEXT,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_notify_filters_group_id ON notify_filters (group_id);
CREATE INDEX idx_notify_filters_ticker ON notify_filters (ticker);
```

### 決算情報
取得した決算情報を記録する。型定義(`earnings_evaluation`)はこの節で行い、`notify_queue`側では再定義せず使い回す。
```sql
-- 決算評価
CREATE TYPE earnings_evaluation AS ENUM (
    'POSITIVE',
    'NEUTRAL',
    'NEGATIVE',
    'UNRATED'
);

-- ソース
CREATE TYPE earnings_source AS ENUM (
    'kabuyoho'
);

-- 決算情報
CREATE TABLE earnings (
  -- 自動採番ID
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  -- 証券コード
  ticker TEXT NOT NULL,
  -- 会社名
  company_name TEXT NOT NULL,
  -- 公開時間
  published_at TIMESTAMPTZ NOT NULL,
  -- 決算タイトル
  title TEXT NOT NULL,
  -- 詳細URL
  url TEXT NOT NULL,
  -- 内容の要約等
  summary TEXT,
  -- 決算評価
  evaluation earnings_evaluation NOT NULL,
  -- 新規判別用Fingerprint
  fingerprint TEXT NOT NULL UNIQUE,
  -- 取得元サイト
  source earnings_source NOT NULL
);

CREATE INDEX idx_earnings_published_at ON earnings (published_at DESC);
CREATE INDEX idx_earnings_ticker ON earnings (ticker);
```

### 送信内容
monitor処理で取得した、notify処理で送信すべき内容を入れるテーブル。ソースなどは重複するが非正規化した方が楽なので非正規化する。monitor処理をするたびに削除・変更される。`earnings`テーブルとの対応を追跡できるよう`fingerprint`を参照として持たせる。
```sql
CREATE TYPE notify_status AS ENUM (
  'ready',  -- 送信準備完了
  'sent',   -- 送信済み
  'failed'  -- 送信失敗
);

CREATE TABLE notify_queue (
  -- 自動採番ID
  id BIGSERIAL PRIMARY KEY,
  -- earningsテーブルとの対応関係を追跡するための参照
  fingerprint TEXT NOT NULL REFERENCES earnings(fingerprint),
  -- 取得元サイト
  source earnings_source NOT NULL,
  -- 取得時間(スクレイピング開始時間)
  fetched_at TIMESTAMPTZ NOT NULL,
  -- 証券コード
  ticker TEXT NOT NULL,
  -- 銘柄名
  company_name TEXT NOT NULL,
  -- 公開時刻
  published_at TIMESTAMPTZ NOT NULL,
  -- 決算タイトル
  title TEXT NOT NULL,
  -- 詳細情報へのurl
  url TEXT NOT NULL,
  -- 内容の要約など
  summary TEXT NOT NULL,
  -- 決算評価
  evaluation earnings_evaluation NOT NULL,
  -- 送信したか否か等
  status notify_status NOT NULL DEFAULT 'ready'
);

CREATE INDEX idx_notify_queue_status ON notify_queue (status);
```

## コンテナ設計
コンテナは以下の3つ。
- Frontend
- Backend
- DB

ステージは以下の2つ。
- dev
- prod

管理は以下の3ファイル。
- compose.yaml
- compose.dev.yaml
- compose.prod.yaml

Dockerfileは以下の2つ。
- Backend/Dockerfile
- Frontend/Dockerfile

開発中はvite、本番はnginxを使って通信する。Rustでconfigを上書きするenvはcomposeの時点で読み込む。

Python(Playwright)環境は別コンテナに分けず、Rustバイナリが動くBackendコンテナ内に同居させる。Backend/Dockerfile 1つでRustビルド成果物とPython/Playwright依存(Chromiumなど)を両方含める。

---

# 考えなければいけない点(todo)

## 機能アイデア
- 決算情報一覧のCSV出力(ユーザが各々で情報処理できるように。n件ダウンロードなど)

## API設計
ブレスト段階でも軽く考える(何をやるか、どんなパスにするかくらい)。

## パスワードの通信
パスワード・ユーザ名は通信の傍受に耐えられるよう、復元可能なハッシュなどを使って通信することを検討する(個人用途のため優先度は低い)。

## DB設計系
- `notify_discord_configs.embed_color`のフォーマット(16進カラーコード文字列か、Discord APIが求める整数値か)
- Slack/Emailの設定項目の具体化(調査してから)

## CSVの数式コピー対応
`=TEXT(A1,"")`のような数式で列を定義していた場合への対応

## 管理者ユーザとそれ以外のユーザの権限差
`users.role`カラムはあるが、「管理者のみができること」(定期実行のロガー設定、CLIでの`create-admin`など)以外の機能制限がどこまであるか未整理。個人・友人・家族向けなので、シンプルな2階層(admin/user)で足りるか確認する。

## マイグレーション実行のタイミング
`cli migration`は「初期化時に実行する」とあるが、今後スキーマ変更が発生した際の運用フロー(自動実行か、手動でexecして都度流すか)を軽くメモしておく。

## ダッシュボード画面の具体的な表示内容
「何銘柄を監視してるか」以外に、ダッシュボードで何を見せたいか(直近の送信履歴、エラー状況など)が未決定。

## その他の未決定事項(既存メモより)
- `EarningItems`の判別用フィールドを`payload`(結合済み文字列)で持つか、個別フィールドで持つかは実装時に決定
- `server`側の`MemoryLayer`flushトリガー(即時か短時間窓か)は未決定