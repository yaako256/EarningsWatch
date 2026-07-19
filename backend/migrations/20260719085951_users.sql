/*
backend/migrations/20260719085951_users.sql
ユーザテーブルの定義
*/

-- テーブル定義
CREATE TABLE users (
    -- ユーザID
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    -- ユーザ名
    username        TEXT        NOT NULL,
    -- パスワード(Hash)
    password_hash   TEXT        NOT NULL,
    -- ロール
    role            TEXT        NOT NULL DEFAULT 'user',
    -- 作成時刻
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- 更新時刻
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- 削除時刻
    disabled_at     TIMESTAMPTZ
);

-- 制約条件
CREATE UNIQUE INDEX users_username_unique ON users(username);