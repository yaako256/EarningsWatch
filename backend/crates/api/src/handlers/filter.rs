/*
backend/crates/api/src/handlers/filter.rs
フィルタ系のハンドラ

一括操作(bulk-enable/bulk-disable/bulk-delete)は、
app::bulk_filter_actionが所有者チェックを行わない設計のため、
ハンドラ側で対象filter_idsが認証ユーザ配下のグループに属することを事前に絞り込む。
*/

// 外部クレート
use axum::Json;
use axum::extract::{Path, Query, State};

// 内部ライブラリ
use identity::{FilterId, GroupId};
use serde::{Deserialize, Serialize};

// 自クレート
use crate::error::ApiAppError;
use crate::extractor::AuthUser;
use crate::handlers::common::FilterResponse;
use crate::response::{ApiResponse, Page};
use crate::state::AppState;

// ─── GET /api/groups/{id}/filters ───
#[derive(Deserialize)]
pub struct ListFiltersQuery {
  pub page: u32,
  pub per_page: u32,
}

pub async fn list_filters(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path(group_id): Path<GroupId>,
  Query(query): Query<ListFiltersQuery>,
) -> Result<Json<ApiResponse<Page<FilterResponse>>>, ApiAppError> {
  let filters = app::list_filters(
    state.notify_group_repository.as_ref(),
    state.notify_filter_repository.as_ref(),
    auth_user.user_id,
    group_id,
  )
  .await?;

  // list_by_group_idは全件取得のため、ここでページングを適用する
  // (件数が多くなる想定は薄いため、DB層でのLIMIT/OFFSETまでは行わない簡易実装)
  let total_count = filters.len() as i64;
  let start = ((query.page.saturating_sub(1)) * query.per_page) as usize;
  let items: Vec<FilterResponse> = filters
    .into_iter()
    .skip(start)
    .take(query.per_page as usize)
    .map(FilterResponse::from)
    .collect();

  let total_pages = ((total_count as f64) / (query.per_page as f64)).ceil() as u32;

  Ok(Json(ApiResponse::ok(Page {
    items,
    page: query.page,
    per_page: query.per_page,
    total_count,
    total_pages,
  })))
}

// ─── POST /api/groups/{id}/filters ───
#[derive(Deserialize)]
pub struct CreateFilterRequest {
  pub ticker: String,
  pub company_name: String,
  pub notes: Option<String>,
}

pub async fn create_filter(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path(group_id): Path<GroupId>,
  Json(body): Json<CreateFilterRequest>,
) -> Result<Json<ApiResponse<FilterResponse>>, ApiAppError> {
  let filter = app::create_filter(
    state.notify_group_repository.as_ref(),
    state.notify_filter_repository.as_ref(),
    auth_user.user_id,
    group_id,
    body.ticker,
    body.company_name,
    body.notes,
  )
  .await?;

  Ok(Json(ApiResponse::ok(FilterResponse::from(filter))))
}

// ─── PUT /api/groups/{id}/filters/{filter_id} ───
#[derive(Deserialize)]
pub struct UpdateFilterRequest {
  pub ticker: String,
  pub company_name: String,
  pub notes: Option<String>,
}

pub async fn update_filter(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path((group_id, filter_id)): Path<(GroupId, FilterId)>,
  Json(body): Json<UpdateFilterRequest>,
) -> Result<Json<ApiResponse<FilterResponse>>, ApiAppError> {
  let filter = app::update_filter(
    state.notify_group_repository.as_ref(),
    state.notify_filter_repository.as_ref(),
    auth_user.user_id,
    group_id,
    filter_id,
    body.ticker,
    body.company_name,
    body.notes,
  )
  .await?;

  Ok(Json(ApiResponse::ok(FilterResponse::from(filter))))
}

// ─── PATCH .../enable, /disable, DELETE ... ───
pub async fn enable_filter(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path((group_id, filter_id)): Path<(GroupId, FilterId)>,
) -> Result<Json<ApiResponse<()>>, ApiAppError> {
  app::enable_filter(
    state.notify_group_repository.as_ref(),
    state.notify_filter_repository.as_ref(),
    auth_user.user_id,
    group_id,
    filter_id,
  )
  .await?;
  Ok(Json(ApiResponse::ok(())))
}

pub async fn disable_filter(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path((group_id, filter_id)): Path<(GroupId, FilterId)>,
) -> Result<Json<ApiResponse<()>>, ApiAppError> {
  app::disable_filter(
    state.notify_group_repository.as_ref(),
    state.notify_filter_repository.as_ref(),
    auth_user.user_id,
    group_id,
    filter_id,
  )
  .await?;
  Ok(Json(ApiResponse::ok(())))
}

pub async fn delete_filter(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path((group_id, filter_id)): Path<(GroupId, FilterId)>,
) -> Result<Json<ApiResponse<()>>, ApiAppError> {
  app::delete_filter(
    state.notify_group_repository.as_ref(),
    state.notify_filter_repository.as_ref(),
    auth_user.user_id,
    group_id,
    filter_id,
  )
  .await?;
  Ok(Json(ApiResponse::ok(())))
}

// ─── POST bulk-enable / bulk-disable / bulk-delete ───
#[derive(Deserialize)]
pub struct BulkFilterIdsRequest {
  pub filter_ids: Vec<FilterId>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkFilterActionResponse {
  pub updated_count: u32,
}

/// filter_idsのうち、認証ユーザ配下のグループに属するものだけに絞り込む
/// (app::bulk_filter_actionは所有者チェックを行わないため、ハンドラ側の責務とする)
async fn filter_ids_owned_by_user(
  state: &AppState,
  user_id: identity::UserId,
  filter_ids: Vec<FilterId>,
) -> Result<Vec<FilterId>, ApiAppError> {
  let mut owned = Vec::new();
  for filter_id in filter_ids {
    let Some(filter) = state
      .notify_filter_repository
      .find_by_id(filter_id)
      .await
      .map_err(app::AppError::from)?
    else {
      continue;
    };

    if let Some(group) = state
      .notify_group_repository
      .find_by_id(filter.group_id)
      .await
      .map_err(app::AppError::from)?
    {
      if group.user_id == user_id {
        owned.push(filter_id);
      }
    }
  }
  Ok(owned)
}

pub async fn bulk_enable(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Json(body): Json<BulkFilterIdsRequest>,
) -> Result<Json<ApiResponse<BulkFilterActionResponse>>, ApiAppError> {
  let owned_ids = filter_ids_owned_by_user(&state, auth_user.user_id, body.filter_ids).await?;
  let updated_count = app::bulk_filter_action(
    state.notify_filter_repository.as_ref(),
    owned_ids,
    app::BulkAction::Enable,
  )
  .await
  .map_err(ApiAppError::from)?;
  Ok(Json(ApiResponse::ok(BulkFilterActionResponse {
    updated_count,
  })))
}

pub async fn bulk_disable(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Json(body): Json<BulkFilterIdsRequest>,
) -> Result<Json<ApiResponse<BulkFilterActionResponse>>, ApiAppError> {
  let owned_ids = filter_ids_owned_by_user(&state, auth_user.user_id, body.filter_ids).await?;
  let updated_count = app::bulk_filter_action(
    state.notify_filter_repository.as_ref(),
    owned_ids,
    app::BulkAction::Disable,
  )
  .await
  .map_err(ApiAppError::from)?;
  Ok(Json(ApiResponse::ok(BulkFilterActionResponse {
    updated_count,
  })))
}

pub async fn bulk_delete(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Json(body): Json<BulkFilterIdsRequest>,
) -> Result<Json<ApiResponse<BulkFilterActionResponse>>, ApiAppError> {
  let owned_ids = filter_ids_owned_by_user(&state, auth_user.user_id, body.filter_ids).await?;
  let updated_count = app::bulk_filter_action(
    state.notify_filter_repository.as_ref(),
    owned_ids,
    app::BulkAction::Delete,
  )
  .await
  .map_err(ApiAppError::from)?;
  Ok(Json(ApiResponse::ok(BulkFilterActionResponse {
    updated_count,
  })))
}
