/*
backend/crates/repository/src/notify_filter_repository.rs
通知フィルターテーブルを司るリポジトリ
*/

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use identity::{FilterId, GroupId};
use subscription::NotifyFilter;

// 自クレート
use crate::RepositoryResult;

pub struct FilterCountBreakdown {
  pub total: u32,
  pub unique_ticker_count: u32,
  pub unique_company_name_count: u32,
}

/// 通知フィルターテーブルのリポジトリ型
#[async_trait]
pub trait NotifyFilterRepository: Send + Sync {
  async fn find_by_id(&self, id: FilterId) -> RepositoryResult<Option<NotifyFilter>>;
  async fn list_by_group_id(&self, group_id: GroupId) -> RepositoryResult<Vec<NotifyFilter>>;
  async fn insert(&self, filter: &NotifyFilter) -> RepositoryResult<()>;
  async fn update(&self, filter: &NotifyFilter) -> RepositoryResult<()>;
  async fn delete(&self, id: FilterId) -> RepositoryResult<()>;
  /// CSVインポート(Phase 9)の差分反映(00-overview.md 4章原則4「一新ではなく差分検出・反映」)用に、
  /// 1グループ分のフィルタをまとめて置き換える。実装(infra)側で1トランザクションにまとめる。
  async fn replace_all_for_group(
    &self,
    group_id: GroupId,
    filters: &[NotifyFilter],
  ) -> RepositoryResult<()>;

  /// ユーザ単位のフィルタ数内訳(総数・ユニーク銘柄数×2種)
  async fn count_breakdown_by_user(
    &self,
    user_id: identity::UserId,
  ) -> RepositoryResult<FilterCountBreakdown>;
}
