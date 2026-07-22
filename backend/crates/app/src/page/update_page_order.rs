/*
backend/crates/app/src/page/update_page_order.rs
staticページを並び替えるユースケース
*/

// 内部ライブラリ
use content::{Page, PageKind, PageType};
use identity::PageId;
use repository::PageRepository;

// 自クレート
use crate::AppError;

// 再採番間隔
const RENUMBER_STEP: i32 = 1000;

/// design/03-features/notice-board.md 7章の並べ替えアルゴリズム。
/// 1. 要求された値でまず仮更新する(一意性制約がないため即座にエラーにはならない)
/// 2. static全ページを現在の並び順で取得し直す(仮更新済みのため移動対象も正しい位置に並ぶ)
/// 3. 隣接2件の差が1以下(中間値が取れない)箇所があれば、全体を1000刻みに再採番する
pub async fn update_page_order(
  page_repo: &dyn PageRepository,
  page_id: PageId,
  new_display_order: i32,
) -> Result<Page, AppError> {
  let page = page_repo
    .find_by_id(page_id)
    .await?
    .ok_or(AppError::NotFound)?;

  if !matches!(page.kind, PageKind::Static { .. }) {
    return Err(AppError::InvalidInput(
      "staticページのみ並べ替えできます".to_string(),
    ));
  }

  page_repo
    .update_display_order(page_id, new_display_order)
    .await?;

  let mut pages = page_repo.list_by_type(PageType::Static).await?;
  pages.sort_by_key(|p| match p.kind {
    PageKind::Static { display_order } => display_order,
    PageKind::Blog => i32::MAX,
  });

  let needs_renumber = pages
    .windows(2)
    .any(|pair| match (&pair[0].kind, &pair[1].kind) {
      (PageKind::Static { display_order: a }, PageKind::Static { display_order: b }) => {
        (b - a).abs() <= 1
      }
      _ => false,
    });

  if needs_renumber {
    for (idx, p) in pages.iter().enumerate() {
      let renumbered = (idx as i32 + 1) * RENUMBER_STEP;
      page_repo.update_display_order(p.id, renumbered).await?;
    }
  }

  page_repo
    .find_by_id(page_id)
    .await?
    .ok_or(AppError::NotFound)
}
