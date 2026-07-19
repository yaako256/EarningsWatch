/*
backend/migrations/20260719085821_logging.sql
ロギングテーブルの定義
*/

-- ログの由来プロセス
CREATE TYPE log_process AS ENUM (
    'server',
    'monitor',
    'notify'
);

-- テーブル定義
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
  -- 構造化ログ
  fields      JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- 制約条件
CREATE INDEX idx_logs_timestamp ON logs (timestamp DESC);
CREATE INDEX idx_logs_process ON logs (process);