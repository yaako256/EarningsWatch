/*
backend/crates/repository/src/system_run_repository.rs
システム実行履歴テーブルを司るリポジトリ
*/

// 外部クレート
use async_trait::async_trait;

// 自クレート
use crate::RepositoryResult;

/// システム実行履歴テーブルのリポジトリ型
/// 本書3.4節の決定: ダッシュボード集計はPhase 10で仕様確定後に設計するため、
/// Phase 4時点では「1回の実行結果を記録する」ことのみを対象とする。
#[async_trait]
pub trait SystemRunRepository: Send + Sync {
  async fn record_monitor_run(
    &self,
    run_at: chrono::DateTime<chrono::Utc>,
    duration_ms: i32,
    new_earnings_count: i32,
  ) -> RepositoryResult<()>;

  async fn record_notify_run(
    &self,
    run_at: chrono::DateTime<chrono::Utc>,
    duration_ms: i32,
    total_send_count: i32,
    success_send_count: i32,
  ) -> RepositoryResult<()>;

  async fn recent_notify_success_rate(&self, recent_n: u32) -> RepositoryResult<Option<f64>>;

  /// 最終監視実行時刻
  async fn last_monitor_run_at(&self) -> RepositoryResult<Option<chrono::DateTime<chrono::Utc>>>;

  /// run_typeは生文字列('monitor'/'notify')で返し、api層でSystemRunTypeへ変換する
  async fn recent_run_durations(
    &self,
    recent_n: u32,
  ) -> RepositoryResult<Vec<(String, chrono::DateTime<chrono::Utc>, i32)>>;
}
