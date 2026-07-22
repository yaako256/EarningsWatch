/*
backend/crates/app/src/notify/list_notify_history.rs
送信履歴を返すユースケース
*/

// 内部ライブラリ
use identity::GroupId;
use repository::NotifyHistoryRepository;
use subscription::{NotifyHistoryEntry, NotifyStatus};

// 自クレート
use crate::AppError;

pub async fn list_notify_history(
  history_repo: &dyn NotifyHistoryRepository,
  group_id: Option<GroupId>,
  _status: Option<NotifyStatus>, // Phase 4のRepository Traitがstatus絞り込みを持たないため、本Phaseではgroup_idのみ対応
  page: u32,
  per_page: u32,
) -> Result<(Vec<NotifyHistoryEntry>, i64), AppError> {
  match group_id {
    Some(gid) => Ok(history_repo.list_by_group_id(gid, page, per_page).await?),
    None => Ok(history_repo.list_all(page, per_page).await?),
  }
}
