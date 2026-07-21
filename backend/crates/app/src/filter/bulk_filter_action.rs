/*
backend/crates/app/src/filter/bulk_filter_action.rs
フィルタを一括設定するユースケース
*/

// 内部ライブラリ
use identity::FilterId;
use repository::NotifyFilterRepository;

// 自クレート
use crate::AppError;

pub enum BulkAction {
  Enable,
  Disable,
  Delete,
}

/// 一括操作は所有者チェックを1件ずつ行うと重くなるため、フィルタ自体の存在確認のみ行う。
/// 呼び出し元(ハンドラ)が、対象filter_idsが認証ユーザ配下のグループに属することを
/// 事前に絞り込んでおく前提とする。
pub async fn bulk_filter_action(
  filter_repo: &dyn NotifyFilterRepository,
  filter_ids: Vec<FilterId>,
  action: BulkAction,
) -> Result<u32, AppError> {
  let mut updated_count = 0u32;

  for filter_id in filter_ids {
    let Some(mut filter) = filter_repo.find_by_id(filter_id).await? else {
      continue; // 存在しないIDはスキップ(エラーにしない)
    };

    match action {
      BulkAction::Enable => {
        filter.enabled = true;
        filter_repo.update(&filter).await?;
      }
      BulkAction::Disable => {
        filter.enabled = false;
        filter_repo.update(&filter).await?;
      }
      BulkAction::Delete => {
        filter_repo.delete(filter_id).await?;
      }
    }

    updated_count += 1;
  }

  Ok(updated_count)
}
