/*
backend/migrations/20260719090302_pages.sql
ページのテーブル定義
*/

-- ページ種類の列挙
CREATE TYPE page_type AS ENUM ('blog', 'static');

-- テーブル定義
CREATE TABLE pages (
    -- ページID
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- ページタイプ
    type page_type NOT NULL,
    -- タイトル
    title TEXT NOT NULL,
    -- 内容(markdown)
    content_markdown TEXT NOT NULL,
    -- 表示順番(static)
    display_order INTEGER,
    -- 公開するか
    is_published BOOLEAN NOT NULL DEFAULT TRUE,
    -- 作成日時
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- 更新日時
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- 作成者
    created_by UUID NOT NULL REFERENCES users(id)
);

-- 制約条件
CREATE INDEX idx_pages_type ON pages (type);
CREATE INDEX idx_pages_created_at ON pages (created_at DESC);