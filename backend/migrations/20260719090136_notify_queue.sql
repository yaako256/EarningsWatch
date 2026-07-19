/*
backend/migrations/20260719090136_notify_queue.sql
通知キューテーブル定義
*/

-- 通知statusの列挙
CREATE TYPE notify_status AS ENUM (
  'ready',
  'sent',
  'failed'
);

CREATE TABLE notify_queue (
  -- 自動採番ID
  id BIGSERIAL PRIMARY KEY,
  -- 決算fingerprint
  fingerprint TEXT REFERENCES earnings(fingerprint),
  -- monitor処理用マーカー
  is_monitor_marker BOOLEAN NOT NULL DEFAULT FALSE,
  -- 決算情報のソースラベル
  source earnings_source,
  -- 取得時刻
  fetched_at TIMESTAMPTZ NOT NULL,
  -- 証券コード
  ticker TEXT,
  -- 銘柄名
  company_name TEXT,
  -- 公開時刻
  published_at TIMESTAMPTZ,
  -- 決算タイトル
  title TEXT,
  -- 決算詳細URL
  url TEXT,
  -- 決算要約
  summary TEXT,
  -- 決算評価
  evaluation earnings_evaluation,
  -- 送信status
  status notify_status NOT NULL DEFAULT 'ready'
);

-- 制約条件
CREATE INDEX idx_notify_queue_status ON notify_queue (status);
CREATE INDEX idx_notify_queue_is_monitor_marker ON notify_queue (is_monitor_marker);