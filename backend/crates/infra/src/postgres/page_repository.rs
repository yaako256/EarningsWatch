/*
backend/crates/infra/src/postgres/page_repository.rs
お知らせ板テーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// 内部ライブラリ
use content::{Page, PageKind, PageType};
use identity::{PageId, UserId};
use repository::{PageRepository, RepositoryError, RepositoryResult};

// 自クレート
use crate::error_mapping::{map_conflict_error, map_error};

pub struct PgPageRepository {
  pool: PgPool,
}

impl PgPageRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

struct PageRow {
  id: Uuid,
  r#type: PageType,
  title: String,
  content_markdown: String,
  display_order: Option<i32>,
  is_published: bool,
  created_at: chrono::DateTime<chrono::Utc>,
  updated_at: chrono::DateTime<chrono::Utc>,
  created_by: Uuid,
}

impl TryFrom<PageRow> for Page {
  type Error = RepositoryError;

  fn try_from(row: PageRow) -> Result<Self, Self::Error> {
    let kind = match row.r#type {
      PageType::Blog => PageKind::Blog,
      PageType::Static => PageKind::Static {
        display_order: row
          .display_order
          .ok_or_else(|| RepositoryError::Other("staticページのdisplay_orderがNULLです".into()))?,
      },
    };

    Ok(Page {
      id: PageId::from_uuid(row.id),
      kind,
      title: row.title,
      content_markdown: row.content_markdown,
      is_published: row.is_published,
      created_at: row.created_at,
      updated_at: row.updated_at,
      created_by: UserId::from_uuid(row.created_by),
    })
  }
}

#[async_trait]
impl PageRepository for PgPageRepository {
  async fn find_by_id(&self, id: PageId) -> RepositoryResult<Option<Page>> {
    let row = sqlx::query_as!(
      PageRow,
      r#"
      SELECT id, type as "type: PageType", title, content_markdown, display_order,
        is_published, created_at, updated_at, created_by
      FROM pages WHERE id = $1
      "#,
      id.as_uuid()
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(map_error)?;

    row.map(Page::try_from).transpose()
  }

  async fn list_by_type(&self, page_type: PageType) -> RepositoryResult<Vec<Page>> {
    let rows = sqlx::query_as!(
      PageRow,
      r#"
      SELECT id, type as "type: PageType", title, content_markdown, display_order,
            is_published, created_at, updated_at, created_by
      FROM pages WHERE type = $1
      ORDER BY display_order ASC NULLS LAST, created_at DESC
      "#,
      page_type as PageType
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    rows.into_iter().map(Page::try_from).collect()
  }

  async fn insert(&self, page: &Page) -> RepositoryResult<()> {
    let display_order = match &page.kind {
      PageKind::Static { display_order } => Some(*display_order),
      PageKind::Blog => None,
    };

    sqlx::query!(
      r#"
      INSERT INTO pages (id, type, title, content_markdown, display_order, is_published, created_at, updated_at, created_by)
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
      "#,
      page.id.as_uuid(),
      page.kind.page_type() as PageType,
      page.title,
      page.content_markdown,
      display_order,
      page.is_published,
      page.created_at,
      page.updated_at,
      page.created_by.as_uuid()
    )
    .execute(&self.pool)
    .await
    .map_err(map_conflict_error)?;

    Ok(())
  }

  async fn update(&self, page: &Page) -> RepositoryResult<()> {
    let display_order = match &page.kind {
      PageKind::Static { display_order } => Some(*display_order),
      PageKind::Blog => None,
    };

    sqlx::query!(
      r#"
      UPDATE pages
      SET title = $2, content_markdown = $3, display_order = $4, is_published = $5, updated_at = $6
      WHERE id = $1
      "#,
      page.id.as_uuid(),
      page.title,
      page.content_markdown,
      display_order,
      page.is_published,
      page.updated_at
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }

  async fn delete(&self, id: PageId) -> RepositoryResult<()> {
    sqlx::query!("DELETE FROM pages WHERE id = $1", id.as_uuid())
      .execute(&self.pool)
      .await
      .map_err(map_error)?;

    Ok(())
  }

  async fn update_display_order(&self, id: PageId, display_order: i32) -> RepositoryResult<()> {
    sqlx::query!(
      "UPDATE pages SET display_order = $2 WHERE id = $1",
      id.as_uuid(),
      display_order
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }
}
