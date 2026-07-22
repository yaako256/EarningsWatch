/*
backend/crates/app/src/page/delete_page.rs
ページを削除するユースケース
*/

// 内部ライブラリ
use identity::PageId;
use repository::PageRepository;

// 自クレート
use crate::AppError;

pub async fn delete_page(page_repo: &dyn PageRepository, page_id: PageId) -> Result<(), AppError> {
  page_repo
    .find_by_id(page_id)
    .await?
    .ok_or(AppError::NotFound)?;
  page_repo.delete(page_id).await?;
  Ok(())
}
