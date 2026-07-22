/*
backend/crates/api/src/handlers/notify_history.rs
送信履歴系のハンドラ
*/

// 外部クレート
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

// 内部ライブラリ
use identity::GroupId;
use subscription::NotifyStatus;

// 自クレート
use crate::error::ApiAppError;
use crate::extractor::AuthUser;
use crate::handlers::common::{NotifyHistoryResponse, enrich_notify_history};
use crate::response::{ApiResponse, Page};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListNotifyHistoryQuery {
  pub group_id: Option<GroupId>,
  pub status: Option<NotifyStatus>,
  pub page: u32,
  pub per_page: u32,
}

pub async fn list_notify_history(
  State(state): State<AppState>,
  _auth_user: AuthUser,
  Query(query): Query<ListNotifyHistoryQuery>,
) -> Result<Json<ApiResponse<Page<NotifyHistoryResponse>>>, ApiAppError> {
  let (entries, total_count) = app::list_notify_history(
    state.notify_history_repository.as_ref(),
    query.group_id,
    query.status,
    query.page,
    query.per_page,
  )
  .await?;

  let items = enrich_notify_history(state.notify_group_repository.as_ref(), entries).await?;
  let total_pages = ((total_count as f64) / (query.per_page as f64)).ceil() as u32;

  Ok(Json(ApiResponse::ok(Page {
    items,
    page: query.page,
    per_page: query.per_page,
    total_count,
    total_pages,
  })))
}
