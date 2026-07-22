/*
backend/crates/app/src/page/update_page.rs
ページを更新するユースケース
*/

// 内部ライブラリ
use content::Page;
use identity::PageId;
use repository::PageRepository;

// 自クレート
use crate::AppError;

pub async fn update_page(
  page_repo: &dyn PageRepository,
  page_id: PageId,
  title: String,
  content_markdown: String,
  is_published: bool,
) -> Result<Page, AppError> {
  let mut page = page_repo
    .find_by_id(page_id)
    .await?
    .ok_or(AppError::NotFound)?;

  let title = title.trim().to_string();
  if title.is_empty() {
    return Err(AppError::InvalidInput(
      "titleを入力してください".to_string(),
    ));
  }

  page.title = title;
  page.content_markdown = content_markdown;
  page.is_published = is_published;
  page.updated_at = chrono::Utc::now();

  page_repo.update(&page).await?;
  Ok(page)
}
