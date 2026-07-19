/*
backend/crates/repository/src/page_repository.rs
ページテーブルを司るリポジトリ型
*/

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use content::{Page, PageType};
use identity::PageId;

// 自クレート
use crate::RepositoryResult;

/// ページテーブルのリポジトリ型
#[async_trait]
pub trait PageRepository: Send + Sync {
  async fn find_by_id(&self, id: PageId) -> RepositoryResult<Option<Page>>;
  async fn list_by_type(&self, page_type: PageType) -> RepositoryResult<Vec<Page>>;
  async fn insert(&self, page: &Page) -> RepositoryResult<()>;
  async fn update(&self, page: &Page) -> RepositoryResult<()>;
  async fn delete(&self, id: PageId) -> RepositoryResult<()>;
  /// 並べ替え(design/03-features/notice-board.md 7章、1000刻み+中間値挿入方式)用。
  /// 具体的な再計算アルゴリズムの置き場所はPhase 10で確定する。
  async fn update_display_order(&self, id: PageId, display_order: i32) -> RepositoryResult<()>;
}
