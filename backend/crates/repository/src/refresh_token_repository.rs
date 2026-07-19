/*
backend/crates/repository/src/refresh_token_repository.rs
リフレッシュトークンテーブルのリポジトリ型
*/

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use auth::RefreshToken;
use identity::{RefreshTokenId, UserId};

// 自クレート
use crate::RepositoryResult;

/// リフレッシュトークンテーブルのリポジトリ型
#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
  async fn find_by_token_hash(&self, token_hash: &str) -> RepositoryResult<Option<RefreshToken>>;
  async fn list_by_user_id(&self, user_id: UserId) -> RepositoryResult<Vec<RefreshToken>>;
  async fn insert(&self, token: &RefreshToken) -> RepositoryResult<()>;
  async fn revoke(&self, id: RefreshTokenId) -> RepositoryResult<()>;
  /// ログアウト時、当該ユーザの有効なリフレッシュトークンを全て失効させる場合に使う
  async fn revoke_all_for_user(&self, user_id: UserId) -> RepositoryResult<()>;
}
