/*
backend/crates/repository/src/notify_queue_repository.rs
送信キューテーブルを司るリポジトリ
*/

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use subscription::{NotifyQueueEntry, NotifyStatus};

// 自クレート
use crate::RepositoryResult;

/// 送信キューテーブルのリポジトリ型
#[async_trait]
pub trait NotifyQueueRepository: Send + Sync {
  /// monitor開始時のマーカー行挿入(01-db-schema.md 6章「monitorの処理順序」1.)
  async fn insert_monitor_marker(&self) -> RepositoryResult<()>;
  /// monitor完了時のマーカー行削除(同3.)
  async fn delete_monitor_marker(&self) -> RepositoryResult<()>;
  /// notify実行開始時の健全性チェック(同「monitor健全性チェック」)。マーカー行が存在すればtrue
  async fn monitor_marker_exists(&self) -> RepositoryResult<bool>;

  /// 既存の決算データ行を削除し、新規分をreadyで一括追加する(monitorの処理順序3.)
  async fn replace_data_rows(&self, entries: &[NotifyQueueEntry]) -> RepositoryResult<()>;
  /// notify実行時、ready状態の決算データ行を全件取得する
  async fn list_ready(&self) -> RepositoryResult<Vec<NotifyQueueEntry>>;
  async fn update_status(&self, id: i64, status: NotifyStatus) -> RepositoryResult<()>;
}
