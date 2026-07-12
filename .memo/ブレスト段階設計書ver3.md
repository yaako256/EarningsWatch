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
migration:    SQLのマイグレーション。未適用のマイグレーションファイルのみ差分適用する
monitor:      監視処理(スクレイピング処理)をする
notify:       送信処理をする
```
スクレイピング(30分前など)と送信を別コマンド・別タイミングで実行することで、送信時間が実行時間によってブレないようにする。

`migration`はマイグレーションツール(sqlx)の管理テーブルにより「どこまで適用済みか」を記録するため、何度実行しても未適用分のみが適用される(差分がなければ何もしない)。この性質を利用し、開発中と本番でそれぞれ以下のタイミングで実行する。

- 開発中: スキーマファイルを変更したタイミングで、手動で`cli migration`を実行
- 本番: コンテナ起動時に必ず実行(後述の`entrypoint.sh`を参照)


## スクレイピング設計(一覧→個別ページ問題)

サイトは「一覧→個別ページ」と遷移する必要があるが、決算件数が多いため個別ページへ毎回遷移するのはコストが高い。そのため、一覧ページの段階で新規/既知を判別し、個別ページへは新規分のみ遷移する。

### 差分検出方式: fingerprint
一覧ページで取得できる情報(証券コード・公開日時・決算タイトルなど)のみを使い、専用構造体からハッシュ化した値を`fingerprint`として新規/既知の判別キーとする。

- fingerprintの生成(結合・正規化・ハッシュ化)は**Rust側の1つの関数に集約**する(表記揺れによる重複判定防止のため)
- 個別ページ側ではfingerprintを再計算しない。一覧取得時点で確定する不変な識別子として扱う
- DBには決算情報の履歴と並行してfingerprint(ハッシュ化後文字列)を保存する

### tickerの正規化ルール
証券コードは取引所によって接尾辞(`.T`など)の有無が異なるため、**常に接尾辞を除去した形に統一**して扱う(例: `7203.T` → `7203`)。将来的に東証以外の取引所へ対応する可能性を考慮し、接尾辞付きの形式には寄せない。

- DB保存・fingerprint生成・表示のすべてにおいて、この正規化後の値を使う
- ユーザがCSV/フロントエンドから`.T`付きでtickerを入力した場合も、同様に正規化してから保存する(表記揺れによる重複登録を防ぐ)
- 正規化処理は、fingerprint生成と同じくRust側の1関数に集約する

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

#### 新規判定用(サイト固有・monitorクレート内のサイト専用モジュールでのみ持つ・個別フィールドの具体的な内容はサイトごとのモジュール内に閉じる)
```rust
pub struct ItiranScraperOutput {
  pub items: Vec<EarningItems>,
}

pub struct EarningItems {
  // 判別用の生データを個別フィールドで持つ(payload結合済み文字列ではなく個別フィールド方式に決定)
  // フィールド内容(証券コード・公開日時・タイトル等、どの項目を持つか)はスクレイピング対象サイトごとに異なるため、
  // 各サイト専用モジュール内で実装時に決定する
  // 結合・正規化(ticker正規化含む)・ハッシュ化はRust側の1関数に集約する方針は決定済み
  // 以下は例。実装時に決定
  pub fingerprint_item_1: String,
  pub fingerprint_item_2: String,
  pub fingerprint_item_3: String,
  pub fingerprint_item_4: String,

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
  - `server`: 1分程度(config管理)の短時間窓でまとめてflush。同一時間窓内に複数件のwarn/errorが発生した場合、まとめて1回の通知として送信する

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

### ファイル形式ごとの数式対応方針
| 形式 | 方針 |
| --- | --- |
| CSV | 値のみを扱う。`=`始まりの数式文字列が含まれていた場合は「CSVでは数式はサポートしていません。Excel(.xlsx)をご利用ください。」というエラーにする |
| Excel(.xlsx) | 数式に対応する。数式セルは計算結果(値)を取得してJSONへ変換する |

### 責務分担
| 層 | 責務 |
| --- | --- |
| フロントエンド | CSV/xlsxの違いを吸収し共通のJSON配列に変換する。数式エラー値(`#REF!`等)の検知、ヘッダ不足など、ファイル・パース由来のエラーをここで弾く |
| バックエンド(二重チェック) | `=`始まり文字列が紛れ込んでいないかの最低限の検知(直接APIアクセス等への保険) |
| バックエンド(業務ルール) | 「明らかにおかしい値」の異常値検知(閾値は本設計時に決定)、必須列の空チェック、重複チェック、GroupNameの自動作成/無効化判定など |

ファイル解析・数式処理そのものはフロントエンド側の責務とし、バックエンド(Rust)はCSV/Excelの違いを意識せず、変換済みのJSONを受け取って処理する。

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
想定画面は以下の3つ。

- 設定系
  - 全体設定
  - グループ毎の設定(送信先/送信先ごとの設定など)
  - グループ毎の送信フィルタ
- ダッシュボード(ユーザ単位の情報)
- 決算情報系(決算情報ログ・CSV出力、決算集中度などのグラフ)

### ダッシュボード表示内容(ユーザ単位)
| 表示項目 | 内容 |
| --- | --- |
| グループ数 | 自分が持つ`notify_groups`の件数 |
| フィルタ数(3行) | 総フィルタ数、ユニーク銘柄数(証券コード基準/銘柄名基準の重複排除) |
| 送信媒体ごとの数 | discord/slackそれぞれのグループ数 |
| 直近n件の送信(全体) | `notify_history`から新しい順にn件 |
| 直近n件の送信(グループごと) | グループを選択して絞り込み表示 |
| 一時停止中のグループ数 | `paused_at IS NOT NULL`の件数 |
| webhook未設定のグループ数 | `notify_discord_configs.webhook_url IS NULL`等の件数 |
| 直近の送信失敗 | `notify_history`の`status = failed`を新しい順に数件 |

システム全体の実績(累計スクレイピング件数、送信成功率、最終監視実行時刻など)はユーザ単位の情報ではないため、ここには含めず管理者専用ダッシュボードとして別画面に切り出す。

### 決算情報系画面
決算情報は株の情報であり全ユーザ共通で見られるものなので、ダッシュボードとは別画面として独立させる。

- 決算情報ログ(CSV出力可、ticker/company_name/evaluation/日付でフィルタ)
- 決算集中度などのグラフ(`earnings`テーブルの`published_at`を日別集計して表示)

### 管理者専用ダッシュボード(新設)
システム全体の実績・稼働状況を見る画面。ユーザダッシュボードとは完全に切り離す。

| 表示項目 | 内容 |
| --- | --- |
| 累計スクレイピング件数 | `earnings`の`COUNT(*)`(または`system_runs.new_earnings_count`の`SUM()`) |
| 送信成功率 | `system_runs`(`run_type='notify'`)の直近n件から`SUM(success_send_count) / SUM(total_send_count)` |
| 最終監視実行時刻 | `system_runs`(`run_type='monitor'`)の`run_at`を最新1件取得 |
| 実行時間の推移 | `run_type`ごとに`duration_ms`を時系列で表示 |

### 対応環境
スマホ用UIは用意しない。スマホ(モバイル幅)でアクセスした場合は「スマホに対応していません。PCで開いてください。」という案内画面を表示する。
それでも開きますか？という表示を出して、開けるようにもできるようにする。
将来拡張として、スマホに対応したレスポンシブCSSを用意することを検討する。

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
CREATE TYPE notify_medium AS ENUM ('discord', 'slack');

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
    paused_at TIMESTAMPTZ,  -- NULL以外なら一時停止中
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Discord固有設定(グループ作成時に必ず1行作成し、以後残し続ける)
CREATE TABLE notify_discord_configs (
    group_id UUID PRIMARY KEY REFERENCES notify_groups(id) ON DELETE CASCADE,
    webhook_url TEXT,  -- NULL許容。未設定時はDiscord送信を試みない(暗号化して保存)
    embed_color TEXT,  -- 16進カラーコード文字列(例:0x87EB87)
     mention_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    mention_targets TEXT[] NOT NULL DEFAULT '{}'  -- 複数指定可。ユーザID/ロールID/"everyone"等
);

-- Slack固有設定(フィールド未定、仮カラム)(後で調べる)
CREATE TABLE notify_slack_configs (
    group_id UUID PRIMARY KEY REFERENCES notify_groups(id) ON DELETE CASCADE,
    webhook_url TEXT  -- NULL許容。未設定時はSlack送信を試みない(暗号化して保存)
    mention_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    mention_targets TEXT[] NOT NULL DEFAULT '{}'  -- 複数指定可。ユーザID/"@here"/"@channel"等
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

### 送信履歴
`notify_queue`とは異なり、実行のたびに削除されない永続的な配送ログ。1グループ×1決算の送信につき1行。「いつ・どのグループへ・何を送ったか」を追跡する。`earnings`のカラムは非正規化せず`fingerprint`経由でJOINして参照する。
```sql
CREATE TABLE notify_history (
  -- 自動採番ID
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  -- 送信先グループ
  group_id UUID NOT NULL REFERENCES notify_groups(id) ON DELETE CASCADE,
  -- earningsテーブルとの対応関係を追跡するための参照
  fingerprint TEXT NOT NULL REFERENCES earnings(fingerprint),
  -- 送信時刻
  sent_at TIMESTAMPTZ NOT NULL,
  -- 送信結果
  status notify_status NOT NULL
);

CREATE INDEX idx_notify_history_group_id ON notify_history (group_id);
CREATE INDEX idx_notify_history_sent_at ON notify_history (sent_at DESC);
```

### システム実績(管理者専用ダッシュボード用)
`monitor`/`notify`の実行ごとに1行記録する。累計スクレイピング件数は`earnings`の`COUNT(*)`(または本テーブルの`new_earnings_count`の`SUM()`)で都度集計するため、別途カウンタ列は持たせない。
```sql
CREATE TYPE run_type AS ENUM ('monitor', 'notify');

CREATE TABLE system_runs (
  -- 自動採番ID
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  -- どちらの実行由来か
  run_type run_type NOT NULL,
  -- 実行時刻
  run_at TIMESTAMPTZ NOT NULL,
  -- 実行時間(ミリ秒)
  duration_ms INT NOT NULL,
  -- monitor由来のみ。今回新規に捕捉した決算件数
  new_earnings_count INT,
  -- notify由来のみ。今回の総送信数
  total_send_count INT,
  -- notify由来のみ。今回の送信成功数
  success_send_count INT
);

CREATE INDEX idx_system_runs_run_type_run_at ON system_runs (run_type, run_at DESC);
```

`monitor`実行時は`new_earnings_count`と`duration_ms`のみ記入し、`notify`実行時は`total_send_count`/`success_send_count`と`duration_ms`のみ記入する(該当しない列はNULL)。

## 管理者ユーザとそれ以外の権限差

個人・家族・友人向けの小規模運用のため、シンプルな2階層(admin/user)とする。グループ数・フィルタ数などの利用制限は設けない(全ユーザ共通で無制限)。

### 管理者(admin)ができること
| 機能 | 内容 | 対応API |
| --- | --- | --- |
| ログ確認 | 全ユーザ分のログを閲覧(日時範囲・ページング) | `GET /api/admin/logs` |
| ユーザ確認 | ユーザ一覧・各ユーザの利用状況(グループ数/フィルタ数/媒体種別)を閲覧 | `GET /api/admin/users`, `GET /api/admin/users/{id}/summary` |
| 仮ユーザ作成 | username指定+ランダム仮パスワード自動生成でユーザを作成 | `POST /api/admin/users` |
| ユーザ無効化(BAN) | 問題があった場合にユーザを無効化(`disabled_at`セット) | `POST /api/admin/users/{id}/disable` |
| 定期実行ロガーの通知先設定 | warn/error通知先(Discord等)の設定 | `GET/PUT /api/admin/notify-config` |

### 一般ユーザ(user)ができること
| 機能 | 内容 | 対応API |
| --- | --- | --- |
| 自分のユーザ名・パスワード変更 | 仮アカウントから本アカウントへの移行(任意、強制しない) | `PUT /api/users/me/username`, `PUT /api/users/me/password` |
| グループ・フィルタ・送信先の管理 | 制限なし | 各種グループ/フィルタAPI |

### 管理者が閲覧できない情報
フィルタの中身(ticker/company_name等)は個人の監視対象情報であるため、管理者であっても閲覧不可。集計値(グループ数・フィルタ数・媒体種別)のみ閲覧可能とする。

### 仮ユーザ作成フロー
1. 管理者がフロントエンドで仮ユーザ作成を実行(usernameは管理者が指定、パスワードはランダム自動生成)
2. 生成された仮パスワードは管理者画面に**一度だけ表示**(再表示不可。DBにはハッシュのみ保存)
3. 管理者が口頭・チャット等でユーザ本人に仮パスワードを伝える
4. ユーザ名・パスワードの変更は**任意**(強制しない。変えなくても使えるが、変えたくなる程度にはランダムな仮パスワードとする)


## API設計

APIはJSONベースとし、レスポンスは成功・失敗ともにエンベロープ形式で返す(YaakoDrive方式を踏襲)。

### レスポンス形式
```json
{ "data": { }, "error": null }
```

```json
{ "data": null, "error": { "code": "not_found", "message": "指定されたリソースが存在しません" } }
```

### エラーコード
| code | HTTPステータス | 意味 |
|------|---------------|------|
| unauthorized | 401 | 未認証 |
| forbidden | 403 | 権限不足 |
| not_found | 404 | リソースが存在しない |
| already_exists | 409 | 重複(ユーザ名など) |
| invalid_request | 422 | リクエスト内容が不正 |
| notify_config_missing | 422 | 送信先設定(webhook_url等)が未設定 |
| notify_send_failed | 502 | 送信先への通信自体が失敗(タイムアウト・DNS等) |
| notify_rejected | 502 | 送信先が非2xxを返した(送信先のHTTPステータスをmessageに含める) |
| import_empty | 422 | インポート対象の行が1件もない(グループ単位インポート時) |
| internal_error | 500 | サーバ内部エラー |

`notify_send_failed`/`notify_rejected`は、送信処理自体の失敗をユーザに詳しく伝える方針とする。messageには具体的な失敗理由(タイムアウトか、名前解決失敗か、送信先が返したHTTPステータスコードなど)を含める。ただし「200 OKだが目的通りに届いていない」(送信先側の設定ミス等)はユーザ側の責任範囲とし、API側では感知しない。

### ページングの共通仕様(オフセット型)
ログ・ユーザ一覧・決算情報・フィルタなど、一覧系のエンドポイントで共通のクエリ・レスポンス形式を使い回す。

**リクエスト**
```
GET /api/xxx?page=1&per_page=50
```

**レスポンス(`data`の中身)**
```json
{
  "items": [ ... ],
  "page": 1,
  "per_page": 50,
  "total_count": 1234,
  "total_pages": 25
}
```

Rust側では以下のようなジェネリック構造体でラップし、実装の重複を避ける。

```rust
#[derive(Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub per_page: u32,
    pub total_count: i64,
    pub total_pages: u32,
}
```

将来、ログなど特定テーブルが大規模化してオフセット型の性能がネックになった場合は、そのエンドポイントだけカーソル方式へ個別に切り替えることを検討する。

### 更新系のメソッド方針
フロントエンドは「設定項目をポチポチ変更→保存ボタンで反映」という操作を想定しているため、**保存ボタンを介す設定系は`PUT`で全体更新**を基本とする(画面に表示されている設定値をまとめて送る)。一方、**トグル的にその場で即時反映したい操作(一時停止/再開、有効化/無効化など)は個別の`PATCH`/専用エンドポイント**とする。

### 認証エラー時のフロー
`/api/auth/refresh`が失敗した場合(リフレッシュトークン期限切れ・revoke済みなど)は`401 unauthorized`を返す。フロントエンドはこれを受けたらログイン状態を破棄し、ログイン画面へ遷移する。

### バリデーション方針
| 項目 | ルール |
| --- | --- |
| `ticker` | 空文字不可のみ(パターンチェックはしない) |
| `company_name` | 空文字不可のみ |
| `notes` | 任意(空文字許容) |
| グループ`name` | 空文字不可、文字数上限のみチェック(具体的な上限値・詳細フォーマットは仮設計時に決定) |
| `webhook_url` | 空文字は許容(未設定として扱う)。値がある場合はURL形式チェックのみ |
| `embed_color` | `NULL`ならデフォルト色として判定。フォーマットは16進カラーコード文字列(例: `0x87EB87`)に確定。フロントエンドではこの文字列を直接入力させず、0〜255のRGBスライダー等による視覚的な色選択UIを用意し、選択結果をこの文字列形式に変換して送信する |
| `mention_targets` | 空配列許容(メンションなしとして扱う)。`mention_enabled = true`かつ配列が空の場合は単にメンションなしとして送信する(一応フロントエンドで、その状態で保存できなくする)。Discord/Slackそれぞれの記法への変換(ユーザID/ロールID/`everyone`/`@here`等)は`notify`処理側の実装詳細とし、ブレスト段階では複数人を配列で保持することのみ確定する |

### 機密情報(webhook_url等)の扱い
webhook_urlは知られると第三者が任意に送信できてしまう認証情報(シークレット)であるため、以下の方針とする。

- DBには**アプリ層で暗号化**(AES-GCM等、方式詳細は仮設計時に決定)した状態で保存する
- 暗号鍵はconfigで管理し、環境変数で上書きする
- 復号は`notify`処理(実際に送信するタイミング)、および`GET /api/groups/{id}/config`のレスポンス生成時に行う
- APIレスポンスではマスクせず、復号した値をそのまま返す(ユーザ本人が設定内容を確認できる必要があるため。目視での盗み見は脅威モデルに含めない)

### その他
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/health` | ヘルスチェック |

### 認証API
| メソッド | パス | 説明 |
|---------|------|------|
| POST | `/api/auth/login` | ログイン |
| POST | `/api/auth/refresh` | トークンのリフレッシュ |
| POST | `/api/auth/logout` | ログアウト |
| GET | `/api/auth/me` | 再訪問時の自動ログイン用 |

### 管理者API
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/admin/logs?from=&to=&page=&per_page=` | ログ一覧(日時範囲フィルタ・ページング) |
| GET | `/api/admin/users?page=&per_page=` | ユーザ一覧(ページング) |
| POST | `/api/admin/users` | 仮ユーザ作成(username指定+ランダム仮パスワード自動生成) |
| POST | `/api/admin/users/{id}/disable` | ユーザ無効化 |
| GET | `/api/admin/users/{id}/summary` | 特定ユーザのグループ数/フィルタ数/媒体種別の集計 |
| GET | `/api/admin/notify-config` | 定期実行ロガーの通知先設定取得 |
| PUT | `/api/admin/notify-config` | 定期実行ロガーの通知先設定更新 |
| GET | `/api/admin/dashboard` | 管理者専用ダッシュボード集計 |

### 決算情報API
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/earnings?ticker=&company_name=&evaluation=&from=&to=&page=&per_page=` | 決算情報一覧(各種フィルタ・ページング) |
| GET | `/api/earnings/export` | 決算情報一覧のCSVエクスポート |
| GET | `/api/earnings/summary` | 決算集中度などの集計(グラフ用データ、日別件数など) |

### 送信履歴API
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/notify-queue?status=&page=&per_page=` | 送信状況一覧(今回実行分。ready/sent/failed等でフィルタ) |
| GET | `/api/notify-history?group_id=&status=&page=&per_page=` | 送信履歴一覧(`notify_history`。グループ・ステータスでフィルタ) |

### グループAPI
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/groups` | 自分のグループ一覧 |
| POST | `/api/groups` | グループ作成 |
| PUT | `/api/groups/{id}` | グループ名・媒体の全体更新 |
| DELETE | `/api/groups/{id}` | グループ削除 |
| PATCH | `/api/groups/{id}/pause` | グループの通知を一時停止(`paused_at`セット) |
| PATCH | `/api/groups/{id}/resume` | グループの通知を再開(`paused_at`をNULLに) |
| GET | `/api/groups/{id}/config` | 媒体別設定取得(discord/slack) |
| PUT | `/api/groups/{id}/config` | 媒体別設定の全体更新 |
| POST | `/api/groups/{id}/config/test-send` | 現在の送信先設定でテスト通知を送信 |
| PUT | `/api/groups/bulk-destination` | 送信先一括設定(複数グループへ反映) |

### フィルタAPI(グループ配下ネスト)
フィルタは必ずいずれかのグループに属するため、パスはグループ配下にネストする。

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/groups/{id}/filters?page=&per_page=` | グループ内フィルタ一覧 |
| POST | `/api/groups/{id}/filters` | フィルタ追加 |
| PUT | `/api/groups/{id}/filters/{filter_id}` | フィルタ内容の全体更新(ticker/company_name/notes) |
| PATCH | `/api/groups/{id}/filters/{filter_id}/enable` | フィルタ有効化 |
| PATCH | `/api/groups/{id}/filters/{filter_id}/disable` | フィルタ無効化 |
| DELETE | `/api/groups/{id}/filters/{filter_id}` | フィルタ削除 |
| POST | `/api/filters/import` | フィルタ一括インポート(CSV/Excel、全体設定) |
| POST | `/api/groups/{id}/filters/import` | フィルタ一括インポート(CSV/Excel、グループ単位) |
| GET | `/api/filters/export` | フィルタ内容のCSVエクスポート |

有効化/無効化はその場でトグルする操作性を想定し、`PATCH`の専用エンドポイントとして分離する。

#### CSVインポートの「壊れた行」の定義
- 行全体が空(必須列も含め何も入力されていない) → 無視(エラーにしない)
- 必須列(`Ticker`, `CompanyName`)のうち片方だけ入力されている(不完全な行) → エラー行として扱う

#### 全体一括設定(`POST /api/filters/import`)固有の挙動
ユーザ全体のフィルタ設定を一新するタイプのインポートでは、グループ名(`GroupName`列)に応じて以下を行う。

- 存在しないGroupNameが指定された → 新規グループとして自動作成
- 既存グループのうち、今回のCSVに1件も含まれなかったもの → そのグループを無効化(`paused_at`セット。削除はしない)
- 追加・無効化が発生した内容はレスポンスDTOでユーザに伝える

#### グループ単位の一括インポート(`POST /api/groups/{id}/filters/import`)固有の挙動
- 対象グループはURLの`{id}`で確定しているため、グループ名の判別・自動作成・無効化は行わない
- インポート対象の有効な行が0件だった場合は`import_empty`エラーとし、インポート処理自体を中断する(既存フィルタは変更しない)

#### フィルタ一括インポートのレスポンス例
インポート結果は専用のレスポンスDTOとし、各項目が初期値(0や空配列)でない場合にフロントエンドで注意表示を行う。

```json
{
  "data": {
    "imported_count": 48,
    "skipped_empty_rows": 2,
    "duplicate_count": 3,
    "error_rows": [],
    "created_groups": ["新春決算グループ"],
    "paused_groups": ["旧グループA"]
  },
  "error": null
}
```

`created_groups`/`paused_groups`は全体一括設定でのみ値が入る(グループ単位インポートでは常に空配列)。

### ユーザ設定API
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/users/me/settings` | 自分の設定取得 |
| PUT | `/api/users/me/settings` | 自分の設定の全体更新 |
| PUT | `/api/users/me/password` | パスワード変更(現在のパスワード確認+新パスワード) |
| PUT | `/api/users/me/username` | ユーザ名変更 |

### ダッシュボードAPI
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/dashboard` | ユーザ単位のダッシュボード集計(グループ数/フィルタ数/送信媒体別数/直近送信/一時停止中グループ数/webhook未設定数/直近の送信失敗) |
| GET | `/api/admin/dashboard` | 管理者専用ダッシュボード集計(累計スクレイピング件数/送信成功率/最終監視実行時刻/実行時間推移) |


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


### 起動時のマイグレーション実行(entrypoint.sh)
本番環境ではコンテナ起動時に`server`が直接起動されるため、その前段でマイグレーションを確実に実行する必要がある。Backendコンテナの`ENTRYPOINT`をシェルスクリプトにし、`migration`→`server`を直列実行する。

配置パス: `Backend/entrypoint.sh`

```sh
#!/bin/sh
set -e
cli migration
exec server
```

- `set -e`: migrationが失敗した場合、即座に停止しserverを起動させない
- `exec server`: `server`プロセスにPID 1を引き継がせ、コンテナのシグナル(SIGTERM等)が正しく届くようにする

Dockerfile側では以下のように組み込む。

```dockerfile
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
```

開発環境ではこのentrypointを使わず、`cli migration`をスキーマ変更時に手動実行し、`server`は別途起動する運用とする。



## クレート構造

DDD・クリーンアーキテクチャを意識し、依存関係逆転によって通知媒体・スクレイピング対象サイト・DBの実装を差し替えやすくする。`shared`クレートは作らず、構造体は各責務のクレートに配置し、エラーは伝搬で対処する。

### Workspace構成
```text
backend/
├── Cargo.toml
└── crates/
    ├── server        # HTTPサーバ起動、DI組み立て
    ├── cli           # 管理コマンド、monitor/notify実行
    ├── app           # ユースケース層
    ├── api           # axum HTTP層
    ├── auth          # 認証/JWT/PasswordHasher
    ├── identity      # UserId, GroupId, FilterIdなどのID型
    ├── earnings      # 決算情報のコアドメイン(Earnings, EarningsEvaluation, fingerprint正規化)
    ├── subscription   # 通知フィルタ・グループの業務ルール(NotifyGroup, NotifyFilter, 一時停止/再開)
    ├── notifier       # 通知媒体 Trait + 媒体ごとの実装(discord/slack)
    ├── scraper       # スクレイピング対象 Trait + サイトごとの実装(Python連携、stdout経由のJSON受け渡し)
    ├── repository    # Repository Trait / UnitOfWork Trait
    ├── infra         # PostgreSQL/sqlx実装
    ├── logging       # tracing Layer(SqlLayer/MemoryLayer)
    └── config        # 設定読み込み
```

### 各クレートの役割
| クレート | 役割 |
| --- | --- |
| `server` | HTTPサーバの起動とDI組み立て。`infra`の具象Repository、`notifier`/`scraper`の具象実装、`app`のユースケース、`api`のRouterを組み合わせる |
| `cli` | 管理コマンド(create-admin, migration)、monitor/notify実行。HTTPを経由せず単発実行される |
| `app` | ユースケース層。ログイン、フィルタ管理、CSVインポート、monitor/notifyの実行フローなどを実装。HTTP・axum・SQL・PostgreSQLには依存しない |
| `api` | HTTP層。Routing、Request解析、Response生成、Cookie処理、JWT認証ミドルウェア、エンベロープ形式へのエラー変換を担当 |
| `auth` | JWT生成・検証、PasswordHasher、TokenClaims、Role等 |
| `identity` | `UserId`、`GroupId`、`FilterId`などのID型 |
| `earnings` | 決算情報のコアドメイン。`Earnings`、`EarningsEvaluation`、fingerprintの生成・正規化(ticker正規化含む)ロジックを1関数に集約して持つ |
| `subscription` | 「誰が何を監視しているか」の業務ルール。`NotifyGroup`、`NotifyFilter`、一時停止/再開ルールを担当 |
| `notifier` | 通知媒体のTraitと媒体ごとの実装。`discord.rs`/`slack.rs`のようにモジュールを追加するだけで媒体を追加できる |
| `scraper` | スクレイピング対象のTraitとサイトごとの実装。Pythonプロセスを起動し、`stdout`経由でJSONを受け取る。サイトを追加する場合はモジュールを追加するだけで済む |
| `repository` | Repository Trait / UnitOfWork Trait。SQLやsqlxには依存しない |
| `infra` | Repository/UnitOfWork TraitのPostgreSQL/sqlx実装。SQL・sqlx・PostgreSQL固有処理はここに閉じ込める |
| `logging` | tracingのLayer実装(`SqlLayer`/`MemoryLayer`) |
| `config` | 設定ファイル・環境変数の読み込み |

### 依存関係
```text
server     -> api, app, infra, config, logging
cli        -> app, infra, config, logging, scraper, notifier
api        -> app, auth, identity
app        -> auth, earnings, subscription, notifier, scraper, repository
infra      -> repository, auth, earnings, subscription, identity
repository -> auth, earnings, subscription, identity
subscription -> identity
earnings   -> identity
notifier   -> identity
scraper    -> earnings
auth       -> identity
config, identity, logging -> 外部crateのみ
```

`scraper`が`earnings`に依存するのは、スクレイピングで取得した生データからfingerprintを組み立てる際、`earnings`クレートが持つ正規化ロジック(ticker正規化含む)を利用するため。

### Python連携方式
`scraper`クレート内で、site専用モジュールごとにPythonプロセスを起動し、`stdout`経由でJSONを受け取る。

```rust
// scraper crate内、site専用モジュール(例: kabuyoho.rs)
pub struct KabuyohoScraper;

#[async_trait]
impl ScraperService for KabuyohoScraper {
    async fn fetch_list(&self, page: u32) -> ScraperResult<Vec<RawEarningItem>> {
        let output = tokio::process::Command::new("python3")
            .arg("scripts/kabuyoho/list.py")
            .arg("--page")
            .arg(page.to_string())
            .output()
            .await?;

        let raw: KabuyohoListOutput = serde_json::from_slice(&output.stdout)?;
        Ok(raw.items)
    }
    // fetch_detailも同様
}
```

Python側はサイトごとに`scripts/kabuyoho/`、`scripts/debug/`のようにディレクトリを分け、クレート内のモジュール分割と対応させる。Python側の出力は以下のように`stdout`へJSONを書き出す形で統一する。

```python
print(json.dumps(output, ensure_ascii=False))
```

`debug.rs`(Phase -1のdebug.py呼び出し用)も`scraper`クレート内の1実装として位置づけ、Phase 13で実サイト実装に差し替える。



## 開発フェーズ
```text
Phase 0: リポジトリ土台
Phase 1: Rust workspace土台(Cargo Workspace、crates構成)
Phase 2: config / logging(SqlLayer/MemoryLayer) / health check
Phase 3: DB / sqlx migration(全テーブル一括作成)
Phase 4: domain型 / repository trait
Phase 5: infra実装(sqlx実装)
Phase 6: CLI管理コマンド(create-admin, migration)
Phase 7: 認証API(login/refresh/logout/me、JWT+Cookie)
Phase 8: グループ/フィルタAPI(CRUD、一時停止/再開)
Phase 9: CSV/Excelインポート・エクスポートAPI
Phase 10: ダッシュボード/送信履歴/管理者API
Phase 11: debug.pyを使ったmonitor/notify CLI実装
Phase 12: 統合テスト/運用確認
Phase 13: 実スクレイピング実装(Python/Playwright、debug.pyを本物に置き換え)
Phase 14: フロントエンド本格実装へ移行
```

### Phase 0の内容
各種プロジェクト・周辺ファイルを揃え、コードはまだ書かずに「開発を始められる状態」にする。

- フロントエンドプロジェクト作成
- バックエンドプロジェクト作成(Cargo Workspace土台)
- sqlファイルの配置場所フォルダ作成
- configファイル配置場所フォルダ作成
- Docker系一式(Dockerfile、compose.yaml/compose.dev.yaml/compose.prod.yaml)
- `.gitignore`、`.dockerignore`
- 実行を容易化するMakefile群の整備

`entrypoint.sh`はこの時点では`cli`コマンドが未実装のため、`cli migration`部分をコメントアウトしておく(Phase 6のCLI実装完了後に有効化)。


### debug.pyについて
Phase -1として最初に、固定値を返すダミースクレイパー(`debug.py`)を作成する。実際のスクレイピング処理(Playwrightでの実サイトアクセス)は不安定さ・サイト構造変更の影響を受けやすいため、これを後回しにし、まずは固定値を返すdebug.pyを使って「fingerprint判定→DB保存→notify送信」という一連のパイプラインを先に完成させる。実スクレイピングへの置き換えはPhase 13で行う。

### 「9.9割完成」の基準(Phase 12時点)
発見された軽微な問題はすべて修正済みで、MVPとして実用化できる段階を指す。未発見の細かな不具合は残っている可能性があるが、既知の課題は残さない状態とする。

### フロントエンドは最後
バックエンド(Phase 0〜13)が9.9割完成してから、フロントエンド本格実装(Phase 14)に着手する方針とする。




## 将来拡張とする点(MVPでは作らない)
- 送信フォーマットの複数指定: Discord/Slack共通で、通知メッセージのフォーマット(シンプルテキスト/リッチ表示など)をグループごとに複数から選べるようにする拡張
- Emailを送信媒体として追加
- https化: 現状はTailscale経由でのアクセスに閉じるため必要ない。
将来インターネット公開するプロジェクトの練習としてhttps化してみてもいいかも
ドメインサービスでホスト名を取得し、nginx + Certbot(またはCaddy)でLet's EncryptによるHTTPS化を行う。

---
# メモ
## configファイルなどについて
環境ラベルのようなもの付け、区別させるようにする。
例:
```
config/
├── .env.dev
├── .env.prod
├── config.dev.toml
└── config.prod.toml
```


## パスワードの通信 → 対応不要と判断
パスワード・ユーザ名は通信の傍受に耐えられるよう、復元可能なハッシュなどを使って通信することを検討する(個人用途のため優先度は低い)。
→ これをしようと思ったが、現在はTailscaleに閉じているため、すでに暗号化されているた。そのためアプリ側での処理は行わない。
しかし、練習として将来拡張には記入した。

---
# 考えなければいけない点(todo)
いったん終了