/*
backend/crates/api/src/handlers/dashboard.rs
ダッシュボード系のハンドラ
*/

// 外部クレート
use axum::Json;
use axum::extract::State;
use serde::Serialize;

// 自クレート
use crate::error::ApiAppError;
use crate::extractor::AuthUser;
use crate::handlers::common::{NotifyHistoryResponse, enrich_notify_history};
use crate::response::ApiResponse;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediumBreakdown {
  pub discord: u32,
  pub slack: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardResponse {
  pub group_count: u32,
  pub filter_count: u32,
  pub unique_ticker_count: u32,
  pub unique_company_name_count: u32,
  pub medium_breakdown: MediumBreakdown,
  pub paused_group_count: u32,
  pub webhook_missing_count: u32,
  pub recent_sent: Vec<NotifyHistoryResponse>,
  pub recent_failed: Vec<NotifyHistoryResponse>,
}

pub async fn get_dashboard(
  State(state): State<AppState>,
  auth_user: AuthUser,
) -> Result<Json<ApiResponse<DashboardResponse>>, ApiAppError> {
  let data = app::get_dashboard(
    state.notify_group_repository.as_ref(),
    state.notify_filter_repository.as_ref(),
    state.notify_discord_config_repository.as_ref(),
    state.notify_slack_config_repository.as_ref(),
    state.notify_history_repository.as_ref(),
    &state.dashboard_settings,
    auth_user.user_id,
  )
  .await?;

  let recent_sent =
    enrich_notify_history(state.notify_group_repository.as_ref(), data.recent_sent).await?;
  let recent_failed =
    enrich_notify_history(state.notify_group_repository.as_ref(), data.recent_failed).await?;

  Ok(Json(ApiResponse::ok(DashboardResponse {
    group_count: data.group_count,
    filter_count: data.filter_count,
    unique_ticker_count: data.unique_ticker_count,
    unique_company_name_count: data.unique_company_name_count,
    medium_breakdown: MediumBreakdown {
      discord: data.discord_group_count,
      slack: data.slack_group_count,
    },
    paused_group_count: data.paused_group_count,
    webhook_missing_count: data.webhook_missing_count,
    recent_sent,
    recent_failed,
  })))
}
