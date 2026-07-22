/*
backend/crates/api/src/handlers/notify_queue.rs
送信キュー関連のハンドラ
*/

// 外部クレート
use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use earnings::EarningsEvaluation;
use subscription::NotifyStatus;

// 自クレート
use crate::error::ApiAppError;
use crate::extractor::AuthUser;
use crate::response::{ApiResponse, Page};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListNotifyQueueQuery {
  pub status: Option<NotifyStatus>,
  pub page: u32,
  pub per_page: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyQueueResponse {
  pub id: i64,
  pub ticker: String,
  pub company_name: String,
  pub published_at: chrono::DateTime<chrono::Utc>,
  pub title: String,
  pub evaluation: EarningsEvaluation,
  pub status: NotifyStatus,
}

impl From<subscription::NotifyQueueEntry> for NotifyQueueResponse {
  fn from(e: subscription::NotifyQueueEntry) -> Self {
    Self {
      id: e.id,
      ticker: e.ticker,
      company_name: e.company_name,
      published_at: e.published_at,
      title: e.title,
      evaluation: e.evaluation,
      status: e.status,
    }
  }
}

pub async fn list_notify_queue(
  State(state): State<AppState>,
  _auth_user: AuthUser,
  Query(query): Query<ListNotifyQueueQuery>,
) -> Result<Json<ApiResponse<Page<NotifyQueueResponse>>>, ApiAppError> {
  let (entries, total_count) = app::list_notify_queue(
    state.notify_queue_repository.as_ref(),
    query.status,
    query.page,
    query.per_page,
  )
  .await?;

  let total_pages = ((total_count as f64) / (query.per_page as f64)).ceil() as u32;

  Ok(Json(ApiResponse::ok(Page {
    items: entries.into_iter().map(NotifyQueueResponse::from).collect(),
    page: query.page,
    per_page: query.per_page,
    total_count,
    total_pages,
  })))
}
