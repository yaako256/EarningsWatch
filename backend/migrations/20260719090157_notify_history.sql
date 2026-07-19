/*
backend/migrations/20260719090136_notify_queue.sql
通知履歴テーブル定義
*/

-- テーブル定義
CREATE TABLE notify_history (
  -- 自動採番ID
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  -- グループID
  group_id UUID REFERENCES notify_groups(id) ON DELETE SET NULL,
  -- 決算fingerprint
  fingerprint TEXT NOT NULL REFERENCES earnings(fingerprint),
  -- 送信時刻
  sent_at TIMESTAMPTZ NOT NULL,
  -- 送信ステータス
  status notify_status NOT NULL
);

-- 制約条件
CREATE INDEX idx_notify_history_group_id ON notify_history (group_id);
CREATE INDEX idx_notify_history_sent_at ON notify_history (sent_at DESC);