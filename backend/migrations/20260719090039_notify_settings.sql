/*
backend/migrations/20260719090039_notify_settings.sql
通知設定系のテーブル定義
*/

-- 送信媒体列挙
CREATE TYPE notify_medium AS ENUM ('discord', 'slack');

-- ユーザ設定のテーブル定義
CREATE TABLE user_settings (
    -- ユーザID
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    -- メモ
    memo TEXT,
    -- 更新時刻
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 通知グループごとの設定テーブル定義
CREATE TABLE notify_groups (
    -- グループID
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- ユーザID
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- グループ名
    name TEXT NOT NULL,
    -- 送信媒体
    medium notify_medium NOT NULL,
    -- 一時停止しているか。その時刻
    paused_at TIMESTAMPTZ,
    -- 作成日時
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- 更新日時
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Discordの送信設定
CREATE TABLE notify_discord_configs (
    -- グループid
    group_id UUID PRIMARY KEY REFERENCES notify_groups(id) ON DELETE CASCADE,
    -- webhookURL(暗号化)
    webhook_url TEXT,
    -- Embedの色(例:0x87EB87)
    embed_color TEXT,
    -- メンション機能を有効化するか
    mention_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    -- メンション対象
    mention_targets TEXT[] NOT NULL DEFAULT '{}'
);

-- Slackの送信設定
CREATE TABLE notify_slack_configs (
    -- グループid
    group_id UUID PRIMARY KEY REFERENCES notify_groups(id) ON DELETE CASCADE,
    -- webhookURL(暗号化)
    webhook_url TEXT,
    -- メンション機能を有効化するか
    mention_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    -- メンション対象
    mention_targets TEXT[] NOT NULL DEFAULT '{}'
);

-- 通知フィルターテーブルの定義
CREATE TABLE notify_filters (
    -- 通知フィルタ識別子(ID)
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
   -- グループID
    group_id UUID NOT NULL REFERENCES notify_groups(id) ON DELETE CASCADE,
    -- 証券コード
    ticker TEXT NOT NULL,
    -- 銘柄名
    company_name TEXT NOT NULL,
    -- 備考
    notes TEXT,
    -- フィルターの有効化/無効化
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    -- 作成日時
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 制約条件
CREATE INDEX idx_notify_filters_group_id ON notify_filters (group_id);
CREATE INDEX idx_notify_filters_ticker ON notify_filters (ticker);