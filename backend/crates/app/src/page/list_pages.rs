/*
backend/crates/app/src/page/list_pages.rs
ページ一覧を取得するユースケース
*/

// 内部ライブラリ
use content::{Page, PageType};
use repository::PageRepository;

// 自クレート
use crate::AppError;

pub async fn list_pages(
  page_repo: &dyn PageRepository,
  page_type: PageType,
) -> Result<Vec<Page>, AppError> {
  // ページング(blogのみ)はapi層で行う(フィルタ一覧と同じ方針)
  Ok(page_repo.list_by_type(page_type).await?)
}
