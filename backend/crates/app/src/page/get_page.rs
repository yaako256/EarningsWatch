/*
backend/crates/app/src/page/get_page.rs
ページ取得のユースケース
*/

// 内部ライブラリ
use content::Page;
use identity::PageId;
use repository::PageRepository;

// 自クレート
use crate::AppError;

pub async fn get_page(page_repo: &dyn PageRepository, page_id: PageId) -> Result<Page, AppError> {
  page_repo
    .find_by_id(page_id)
    .await?
    .ok_or(AppError::NotFound)
}
