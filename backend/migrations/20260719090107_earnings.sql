/*
backend/migrations/20260719090107_earnings.sql
決算情報のテーブル定義
*/

-- 決算評価列挙
CREATE TYPE earnings_evaluation AS ENUM (
    'POSITIVE',
    'NEUTRAL',
    'NEGATIVE',
    'UNRATED'
);

-- ソースのサイト列挙
CREATE TYPE earnings_source AS ENUM (
    'kabuyoho'
);

CREATE TABLE earnings (
  -- 自動採番ID
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  -- 証券コード
  ticker TEXT NOT NULL,
  -- 銘柄名
  company_name TEXT NOT NULL,
  -- 公開時刻
  published_at TIMESTAMPTZ NOT NULL,
  -- 決算タイトル
  title TEXT NOT NULL,
  -- 決算情報へのURL
  url TEXT NOT NULL,
  -- 決算要約
  summary TEXT NOT NULL,
  -- 決算評価
  evaluation earnings_evaluation NOT NULL,
  -- 決算識別fingerprint
  fingerprint TEXT NOT NULL UNIQUE,
  -- 決算ソース
  source earnings_source NOT NULL
);

-- 制約条件
CREATE INDEX idx_earnings_published_at ON earnings (published_at DESC);
CREATE INDEX idx_earnings_ticker ON earnings (ticker);