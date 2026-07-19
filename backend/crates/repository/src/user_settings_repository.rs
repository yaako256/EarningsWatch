/*
backend/crates/repository/src/user_settings_repository.rs
ユーザごとの設定のテーブルを司るリポジトリ
*/

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use identity::UserId;
use subscription::UserSettings;

// 自クレート
use crate::RepositoryResult;

/// ユーザごとの設定のテーブルのリポジトリ型
#[async_trait]
pub trait UserSettingsRepository: Send + Sync {
  async fn find_by_user_id(&self, user_id: UserId) -> RepositoryResult<Option<UserSettings>>;
  async fn upsert(&self, settings: &UserSettings) -> RepositoryResult<()>;
}
