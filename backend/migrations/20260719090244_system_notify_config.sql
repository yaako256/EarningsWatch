/*
backend/migrations/20260719090244_system_notify_config.sql
管理者用送信設定のテーブル定義
*/

-- テーブル定義
CREATE TABLE system_notify_config (
    -- 固定ID
    id BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id = TRUE),
    -- 送信媒体
    medium notify_medium NOT NULL DEFAULT 'discord',
    -- webhookURL
    webhook_url TEXT,
    -- メンションを有効化するか
    mention_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    -- メンション対象
    mention_targets TEXT[] NOT NULL DEFAULT '{}',
    -- 更新日時
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);