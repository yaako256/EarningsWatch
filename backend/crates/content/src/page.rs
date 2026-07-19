/*
backend/crates/content/src/page.rs
お知らせ板・固定ページ(pages)のドメイン型を定義
*/

// 外部クレート
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use identity::{PageId, UserId};

/// ページ種類の列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "page_type", rename_all = "lowercase")]
pub enum PageType {
  Blog,
  Static,
}

/// ページ種類の判別Union
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageKind {
  Blog,
  Static { display_order: i32 },
}

impl PageKind {
  pub fn page_type(&self) -> PageType {
    match self {
      Self::Blog => PageType::Blog,
      Self::Static { .. } => PageType::Static,
    }
  }
}

// pagesテーブル1行分の構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
  pub id: PageId,
  pub kind: PageKind,
  pub title: String,
  pub content_markdown: String,
  pub is_published: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub created_by: UserId,
}
