/*
backend/crates/repository/src/system_notify_config_repository.rs
システムログ用の通知設定テーブルを司るリポジトリ
*/

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use subscription::SystemNotifyConfig;

// 自クレート
use crate::RepositoryResult;

/// システムログ用の通知設定テーブルのリポジトリ型
#[async_trait]
pub trait SystemNotifyConfigRepository: Send + Sync {
  /// 常に1行のみ運用(01-db-schema.md 9章)のため引数なしで取得できる
  async fn get(&self) -> RepositoryResult<Option<SystemNotifyConfig>>;
  async fn upsert(&self, config: &SystemNotifyConfig) -> RepositoryResult<()>;
}
