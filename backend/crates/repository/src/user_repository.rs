/*
backend/crates/repository/src/user_repository.rs
ユーザテーブルを司るリポジトリ
*/

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use auth::User;
use identity::UserId;

// 自クレート
use crate::RepositoryResult;

/// ユーザテーブルのリポジトリ型
#[async_trait]
pub trait UserRepository: Send + Sync {
  async fn find_by_id(&self, id: UserId) -> RepositoryResult<Option<User>>;
  async fn find_by_username(&self, username: &str) -> RepositoryResult<Option<User>>;
  async fn list(&self, page: u32, per_page: u32) -> RepositoryResult<(Vec<User>, i64)>;
  async fn insert(&self, user: &User) -> RepositoryResult<()>;
  async fn update(&self, user: &User) -> RepositoryResult<()>;
}
