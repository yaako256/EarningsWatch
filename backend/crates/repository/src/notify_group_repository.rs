/*
backend/crates/repository/src/notify_group_repository.rs
通知グループテーブルを司るリポジトリ
*/

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use identity::{GroupId, UserId};
use subscription::NotifyGroup;

// 自クレート
use crate::RepositoryResult;

/// 通知グループテーブルのリポジトリ型
#[async_trait]
pub trait NotifyGroupRepository: Send + Sync {
  async fn find_by_id(&self, id: GroupId) -> RepositoryResult<Option<NotifyGroup>>;
  async fn list_by_user_id(&self, user_id: UserId) -> RepositoryResult<Vec<NotifyGroup>>;
  /// notify実行時、グループ横断で全件処理するために使う(01-db-schema.md 6章「notify実行時のグループ別フィルタリング」)
  async fn list_all(&self) -> RepositoryResult<Vec<NotifyGroup>>;
  async fn insert(&self, group: &NotifyGroup) -> RepositoryResult<()>;
  async fn update(&self, group: &NotifyGroup) -> RepositoryResult<()>;
  async fn delete(&self, id: GroupId) -> RepositoryResult<()>;
}
