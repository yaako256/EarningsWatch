/*
backend/crates/app/src/notify/list_notify_queue.rs
送信キューを取得するユースケース
*/

// 内部ライブラリ
use repository::NotifyQueueRepository;
use subscription::{NotifyQueueEntry, NotifyStatus};

// 自クレート
use crate::AppError;

pub async fn list_notify_queue(
  queue_repo: &dyn NotifyQueueRepository,
  status: Option<NotifyStatus>,
  page: u32,
  per_page: u32,
) -> Result<(Vec<NotifyQueueEntry>, i64), AppError> {
  Ok(
    queue_repo
      .list_all_data_rows(status, page, per_page)
      .await?,
  )
}
