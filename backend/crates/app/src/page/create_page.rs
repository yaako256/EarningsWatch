/*
backend/crates/app/src/page/create_page.rs
新規ページを作成するユースケース
*/

// 外部クレート
use chrono::Utc;

// 内部ライブラリ
use content::{Page, PageKind, PageType};
use identity::{PageId, UserId};
use repository::PageRepository;

// 自クレート
use crate::AppError;

// インサートできるような自動採番間隔
const RENUMBER_STEP: i32 = 1000;

#[allow(clippy::too_many_arguments)]
pub async fn create_page(
  page_repo: &dyn PageRepository,
  created_by: UserId,
  page_type: PageType,
  title: String,
  content_markdown: String,
  display_order: Option<i32>,
  is_published: bool,
) -> Result<Page, AppError> {
  let title = title.trim().to_string();
  if title.is_empty() {
    return Err(AppError::InvalidInput(
      "titleを入力してください".to_string(),
    ));
  }

  let kind = match page_type {
    PageType::Blog => PageKind::Blog,
    PageType::Static => {
      let order = match display_order {
        Some(v) => v,
        None => {
          // design/03-features/notice-board.md 7章: 既存最大値+1000
          let existing = page_repo.list_by_type(PageType::Static).await?;
          let max = existing
            .iter()
            .filter_map(|p| match p.kind {
              PageKind::Static { display_order } => Some(display_order),
              PageKind::Blog => None,
            })
            .max()
            .unwrap_or(0);
          max + RENUMBER_STEP
        }
      };
      PageKind::Static {
        display_order: order,
      }
    }
  };

  let now = Utc::now();
  let page = Page {
    id: PageId::new(),
    kind,
    title,
    content_markdown,
    is_published,
    created_at: now,
    updated_at: now,
    created_by,
  };

  page_repo.insert(&page).await?;
  Ok(page)
}
