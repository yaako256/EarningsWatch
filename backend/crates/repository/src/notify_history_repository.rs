/*
backend/crates/repository/src/notify_history_repository.rs
通知履歴テーブルを司るリポジトリ
*/

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use identity::GroupId;
use subscription::NotifyHistoryEntry;

// 自クレート
use crate::RepositoryResult;

/// 通知履歴テーブルのリポジトリ型
#[async_trait]
pub trait NotifyHistoryRepository: Send + Sync {
  async fn insert(&self, entry: &NotifyHistoryEntry) -> RepositoryResult<()>;
  async fn list_by_group_id(
    &self,
    group_id: GroupId,
    page: u32,
    per_page: u32,
  ) -> RepositoryResult<(Vec<NotifyHistoryEntry>, i64)>;
  async fn list_all(
    &self,
    page: u32,
    per_page: u32,
  ) -> RepositoryResult<(Vec<NotifyHistoryEntry>, i64)>;
}
