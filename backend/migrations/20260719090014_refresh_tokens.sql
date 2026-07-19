/*
backend/migrations/20260719090014_refresh_tokens.sql
リフレッシュトークンテーブルの定義
*/

-- テーブル定義
CREATE TABLE refresh_tokens (
    -- トークンID
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    -- ユーザID
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- リフレッシュトークン(Hash)
    token_hash  TEXT        NOT NULL,
    -- ユーザAgent
    user_agent  TEXT,
    -- 期限時刻
    expires_at  TIMESTAMPTZ NOT NULL,
    -- 作成時刻
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- 無効化時刻
    revoked_at  TIMESTAMPTZ
);

-- 制約条件
CREATE UNIQUE INDEX refresh_tokens_token_hash_unique ON refresh_tokens(token_hash);