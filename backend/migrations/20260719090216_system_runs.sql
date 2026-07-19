/*
backend/migrations/20260719090216_system_runs.sql
システム実行履歴のテーブル定義
*/

-- 監視対象の実行タイプ
CREATE TYPE run_type AS ENUM ('monitor', 'notify');

-- テーブル定義
CREATE TABLE system_runs (
  -- 自動採番ID
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  -- 実行タイプ
  run_type run_type NOT NULL,
  -- 実行時刻
  run_at TIMESTAMPTZ NOT NULL,
  -- 実行経過時間
  duration_ms INT NOT NULL,
  -- 新規決算情報数(monitor)
  new_earnings_count INT,
  -- 合計送信数(notify)
  total_send_count INT,
  -- 合計送信成功数(notify)
  success_send_count INT
);

-- 制約条件
CREATE INDEX idx_system_runs_run_type_run_at ON system_runs (run_type, run_at DESC);