# EarningsWatch 本設計書 01. DB設計(DDL全体)

> 本ファイルは`仮設計書.md`13章を土台に、`仮設計書からの追加点と変更点.md`・`仮設計書-型定義.md`で発覚したDDL変更(`notify_queue`拡張、`system_notify_config`新設、`pages`/`page_type`新設)、および本設計書作成時に確定した`notify_history.group_id`の`ON DELETE SET NULL`化を反映した最終形である。

## 目次
1. [ロギング](#1-ロギング)
2. [ユーザ](#2-ユーザ)
3. [リフレッシュトークン](#3-リフレッシュトークン)
4. [ユーザごとの通知設定(グループ・フィルタ)](#4-ユーザごとの通知設定グループフィルタ)
5. [決算情報](#5-決算情報)
6. [送信内容(キュー)](#6-送信内容キュー)
7. [送信履歴](#7-送信履歴)
8. [システム実績(管理者専用ダッシュボード用)](#8-システム実績管理者専用ダッシュボード用)
9. [定期実行ロガーの通知先設定(新設)](#9-定期実行ロガーの通知先設定新設)
10. [お知らせ板・固定ページ(新設)](#10-お知らせ板固定ページ新設)
11. [migration方針](#11-migration方針)

---

## 1. ロギング

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

`level`/`process`のドメイン型(`LogLevel`/`LogProcess`)、行全体のドメイン型(`LogEntry`)は`logging`クレートに配置する(`02-types/`には`logging`クレート専用ファイルは設けず、本ファイルと`02-types/api.md`の管理者ログAPIの節を参照)。管理者ログ一覧API(`GET /api/admin/logs`)は`fields`(JSONB)を含めてそのままフロントエンドへ返す。

## 2. ユーザ

既存アプリ YaakoDrive のものを流用する。

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

ユーザの物理削除・退会機能はMVP・将来拡張ともに実装しない。既存の無効化(BAN、`disabled_at`)で十分と判断する。

## 3. リフレッシュトークン

既存アプリ YaakoDrive のものを流用する。

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

## 4. ユーザごとの通知設定(グループ・フィルタ)

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
    webhook_url TEXT,  -- NULL許容。未設定時はDiscord送信を試みない(cryptoクレートで暗号化して保存)
    embed_color TEXT,  -- 16進カラーコード文字列(例:0x87EB87)。NULLならデフォルト色(0x87CEEB、config管理)
    mention_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    mention_targets TEXT[] NOT NULL DEFAULT '{}'  -- user:<id> / role:<id> / everyone / here / time:<style>
);

-- Slack固有設定(フィールド構成は仮。TODO#1によりDiscord実装完了後のMVP内拡張フェーズで再定義する)
CREATE TABLE notify_slack_configs (
    group_id UUID PRIMARY KEY REFERENCES notify_groups(id) ON DELETE CASCADE,
    webhook_url TEXT,  -- NULL許容。未設定時はSlack送信を試みない(暗号化して保存)
    mention_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    mention_targets TEXT[] NOT NULL DEFAULT '{}'
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

補足:
- 処理上は証券コードと銘柄名の片方が埋まっていればいいが、見た目上どちらも必須とする。
- `notify_filters`は同一グループ内で同じ`ticker`+`company_name`が重複登録されてもUNIQUE制約は設けず、登録自体は許可する(一覧画面では`notes`を含む全カラムを常に表示し、`notes`で見分けがつくようにする)。

## 5. 決算情報

```sql
-- 決算評価
CREATE TYPE earnings_evaluation AS ENUM (
    'POSITIVE',
    'NEUTRAL',
    'NEGATIVE',
    'UNRATED'
);

-- ソース(新しいスクレイピング対象サイトを追加する場合はここに値を追加する)
CREATE TYPE earnings_source AS ENUM (
    'kabuyoho'
);

-- 決算情報
CREATE TABLE earnings (
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  ticker TEXT NOT NULL,
  company_name TEXT NOT NULL,
  published_at TIMESTAMPTZ NOT NULL,
  title TEXT NOT NULL,
  url TEXT NOT NULL,
  summary TEXT,
  evaluation earnings_evaluation NOT NULL,
  -- 新規判別用Fingerprint(タイトル・書き出し・決算評価から生成。証券コード・公開時刻は含めない)
  fingerprint TEXT NOT NULL UNIQUE,
  source earnings_source NOT NULL
);

CREATE INDEX idx_earnings_published_at ON earnings (published_at DESC);
CREATE INDEX idx_earnings_ticker ON earnings (ticker);
```

`earnings`テーブルは削除しない方針とし、無期限に保持する(将来データ量が許容できないほど増えた場合の年1回程度の削除バッチは、優先度の非常に低い将来拡張止まりとする)。

決算評価が速報後に変わった場合(例:速報時`Unrated`→後日`Positive`)、fingerprintに評価を含める設計のため別レコード(別fingerprint)として新規決算と同様に検知・再送信される。逆にfingerprintに含まれない項目(タイトル・書き出し文言等)が変わっても検知されないままとなる。これは意図的な仕様である。

## 6. 送信内容(キュー)

monitor処理で取得した、notify処理で送信すべき内容を入れるテーブル。`earnings`テーブルとの対応を追跡できるよう`fingerprint`を参照として持たせる。

**当初設計からの変更点**: monitor実行中の健全性チェックのため`is_monitor_marker`列を追加し、マーカー行(決算データを持たない)を表現できるよう`fingerprint`をNULL許容に変更した。

```sql
CREATE TYPE notify_status AS ENUM (
  'ready',  -- 送信準備完了
  'sent',   -- 送信済み
  'failed'  -- 送信失敗
);

CREATE TABLE notify_queue (
  id BIGSERIAL PRIMARY KEY,
  -- earningsテーブルとの対応関係を追跡するための参照。マーカー行ではNULL
  fingerprint TEXT REFERENCES earnings(fingerprint),
  -- monitor実行中の健全性チェック用マーカー行か否か(true=マーカー行、false=決算データ行)
  is_monitor_marker BOOLEAN NOT NULL DEFAULT FALSE,
  -- 取得元サイト(マーカー行では意味を持たないためNULL許容とする)
  source earnings_source,
  -- 取得時間(スクレイピング開始時間)
  fetched_at TIMESTAMPTZ NOT NULL,
  -- 証券コード・銘柄名等はマーカー行では使用しないためNULL許容とする
  ticker TEXT,
  company_name TEXT,
  published_at TIMESTAMPTZ,
  title TEXT,
  url TEXT,
  summary TEXT,
  evaluation earnings_evaluation,
  status notify_status NOT NULL DEFAULT 'ready'
);

CREATE INDEX idx_notify_queue_status ON notify_queue (status);
CREATE INDEX idx_notify_queue_is_monitor_marker ON notify_queue (is_monitor_marker);
```

> 仮設計書ver1本体では決算データ列(`source`/`ticker`/`company_name`/`published_at`/`title`/`url`/`summary`/`evaluation`)はすべて`NOT NULL`だったが、マーカー行(`is_monitor_marker = true`)がこれらの値を持たないため、本設計書で改めてNULL許容に変更した。決算データ行(`is_monitor_marker = false`)では引き続きアプリケーション側でこれらを必須として扱う。

### monitorの処理順序
1. monitor開始時、`is_monitor_marker = true`の行を1行挿入する(実行中である印。決算データ行には触れない)
2. スクレイピングを最後まで実行し、新規決算を全部収集する
3. 収集完了後、既存の決算データ行(`is_monitor_marker = false`)を削除し、新規分を`status='ready'`で一括追加する
4. 最後にマーカー行(`is_monitor_marker = true`)を削除する(monitor正常完了の印として消す)

### notify_queueの状態遷移(決算データ行)
`notify`完了後は物理削除せず、`status='sent'`にするのみとする(次回monitorが削除するまで残る)。

### monitor健全性チェック(notify実行開始時)
`notify`実行開始時に`is_monitor_marker = true`の行が存在する場合、monitorが正常に完了していない(実行中またはハング)可能性を示す。この場合、送信を行わず数分待ってから再チェックするリトライを複数回試みる。それでも状況が変わらなければmonitorハングと判断し、`MemoryLayer`経由で開発者向けログ通知経路へ警告を送信する。

### 個別送信失敗時のリトライ(notify実行内で完結)
送信が`failed`になった場合、数分待って再送信を試みるリトライを複数回行う。このリトライは当該notify実行内で完結し、次回notify実行には持ち越さない。最終的にリトライしても失敗した場合は`notify_history`に`failed`として記録される。

### notify実行時のグループ別フィルタリング
`notify_queue`は決算単位(グループ横断で共通)で1行持つ設計であり、グループ紐付けの列は持たない。`monitor`実行時は新規に検出された決算すべてを無条件で`notify_queue`へ登録し、`notify`実行時に登録されている全グループを順々に処理し、各グループに紐づく`notify_filters`(ticker/company_name)でフィルタリングしながら送信する。

## 7. 送信履歴

`notify_queue`とは異なり、実行のたびに削除されない永続的な配送ログ。1グループ×1決算の送信につき1行。`earnings`のカラムは非正規化せず`fingerprint`経由でJOINして参照する。

**当初設計からの変更点**: グループが削除されても送信履歴を失わないよう、`group_id`の`ON DELETE CASCADE`を`ON DELETE SET NULL`に変更した(これに伴い`group_id`をNULL許容化する)。

```sql
CREATE TABLE notify_history (
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  -- 送信先グループ。グループ削除後も履歴自体は残すため、削除時はNULLにする
  group_id UUID REFERENCES notify_groups(id) ON DELETE SET NULL,
  -- earningsテーブルとの対応関係を追跡するための参照
  fingerprint TEXT NOT NULL REFERENCES earnings(fingerprint),
  sent_at TIMESTAMPTZ NOT NULL,
  status notify_status NOT NULL
);

CREATE INDEX idx_notify_history_group_id ON notify_history (group_id);
CREATE INDEX idx_notify_history_sent_at ON notify_history (sent_at DESC);
```

> `group_id`が`NULL`になるのは、送信履歴発生後に対象グループが削除された場合のみである。`GET /api/notify-history`のレスポンス(`NotifyHistoryResponse.group_name`、`02-types/api.md`参照)は、グループ削除後は`group_name`が取得できなくなるため`Option<String>`(nullable)として扱う。フロントエンドは`group_name`が`null`の場合「削除済みグループ」のような表示にフォールバックすることを想定する。

## 8. システム実績(管理者専用ダッシュボード用)

`monitor`/`notify`の実行ごとに1行記録する。累計スクレイピング件数は`earnings`の`COUNT(*)`(または本テーブルの`new_earnings_count`の`SUM()`)で都度集計するため、別途カウンタ列は持たせない。

```sql
CREATE TYPE run_type AS ENUM ('monitor', 'notify');

CREATE TABLE system_runs (
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  run_type run_type NOT NULL,
  run_at TIMESTAMPTZ NOT NULL,
  duration_ms INT NOT NULL,
  -- monitor由来のみ
  new_earnings_count INT,
  -- notify由来のみ
  total_send_count INT,
  success_send_count INT
);

CREATE INDEX idx_system_runs_run_type_run_at ON system_runs (run_type, run_at DESC);
```

`monitor`実行時は`new_earnings_count`と`duration_ms`のみ記入し、`notify`実行時は`total_send_count`/`success_send_count`と`duration_ms`のみ記入する(該当しない列はNULL)。

## 9. 定期実行ロガーの通知先設定(新設)

管理者向けAPI(`GET`/`PUT /api/admin/notify-config`)に対応するテーブルとして新設した。「管理者全体で共有する1つの設定」として運用し、管理者ごとの個別設定(複数行化)は将来拡張とする。

```sql
-- 定期実行ロガー(MemoryLayer)の通知先設定。
-- 管理者全体で共有する1行のみを運用する(複数管理者対応は将来拡張)。
CREATE TABLE system_notify_config (
    id BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id = TRUE), -- 常に1行のみを保証するテクニック
    medium notify_medium NOT NULL DEFAULT 'discord',
    webhook_url TEXT,       -- cryptoクレートで暗号化して保存(用途タグ: SystemNotifyWebhookUrlTag)
    mention_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    mention_targets TEXT[] NOT NULL DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

`medium`/`mention_targets`は`notify_discord_configs`/`notify_slack_configs`(4章)と同じ形式を踏襲する。型定義は`subscription`クレート側に`SystemNotifyConfig`として配置する(グループに紐づかないシステム全体の設定のため)。

## 10. お知らせ板・固定ページ(新設)

管理者がマークダウンで自由に書ける「お知らせ・パッチ通知」(時系列)と「固定ページ」(現在の対象サイト、使い方説明書など)を1つのテーブルで共通化する。

```sql
CREATE TYPE page_type AS ENUM ('blog', 'static');

CREATE TABLE pages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    type page_type NOT NULL,
    title TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    -- staticのみ使用。サイドバー表示順(blogは常にNULL)
    display_order INTEGER,
    is_published BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE INDEX idx_pages_type ON pages (type);
CREATE INDEX idx_pages_created_at ON pages (created_at DESC);
```

- DBには素のマークダウン本文をそのまま保存する。HTML変換は表示時にフロントエンド側で実施する(marked.js等)。
- `display_order`には一意性制約を設けない。管理者による並べ替え操作中に値が一時的に重複しても許容し、UI側で最終的な順序を再計算・保存する運用とする(実装時、並べ替えAPIの具体的なアルゴリズムは`03-features/notice-board.md`で扱う)。
- 閲覧権限は全ユーザ共通(ログインしていれば誰でも閲覧可能)。作成・編集・削除・並べ替えは既存の管理者権限判定(is_admin等)をそのまま流用し、専用権限は設けない。

## 11. migration方針

`sqlx migrate`を利用する。マイグレーションファイルは`backend/migrations/`配下に配置する(sqlx標準の慣習に合わせる)。`migration`はsqlxの管理テーブルにより差分適用のみ行う(冪等性の原則、`00-overview.md`4章参照)。
