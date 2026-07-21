/*
backend/crates/api/src/handlers/filter.rs
フィルタ系のハンドラ

一括操作(bulk-enable/bulk-disable/bulk-delete)は、
app::bulk_filter_actionが所有者チェックを行わない設計のため、
ハンドラ側で対象filter_idsが認証ユーザ配下のグループに属することを事前に絞り込む。
*/

// 外部クレート
use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::IntoResponse;

// 内部ライブラリ
use identity::{FilterId, GroupId};
use serde::{Deserialize, Serialize};

// 自クレート
use crate::error::ApiAppError;
use crate::extractor::AuthUser;
use crate::handlers::common::{ExportFormat, FilterResponse, GroupRef};
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

// ─── POST /api/filters/import(全体一括設定) ───
#[derive(Deserialize)]
pub struct ImportFilterRow {
  pub ticker: String,
  pub company_name: String,
  pub group_name: String,
  pub notes: Option<String>,
  pub enabled: Option<bool>,
}

#[derive(Deserialize)]
pub struct ImportFiltersRequest {
  pub rows: Vec<ImportFilterRow>,
  #[serde(default)]
  pub dry_run: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportErrorRow {
  pub row_number: u32,
  pub reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportWarning {
  pub row_number: u32,
  pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFiltersResponse {
  pub imported_count: u32,
  pub skipped_empty_rows: u32,
  pub duplicate_count: u32,
  pub error_rows: Vec<ImportErrorRow>,
  pub created_groups: Vec<GroupRef>,
  pub paused_groups: Vec<GroupRef>,
  pub warnings: Vec<ImportWarning>,
}

impl From<app::ImportFiltersResult> for ImportFiltersResponse {
  fn from(r: app::ImportFiltersResult) -> Self {
    Self {
      imported_count: r.imported_count,
      skipped_empty_rows: r.skipped_empty_rows,
      duplicate_count: r.duplicate_count,
      error_rows: r
        .error_rows
        .into_iter()
        .map(|e| ImportErrorRow {
          row_number: e.row_number,
          reason: e.reason,
        })
        .collect(),
      created_groups: r
        .created_groups
        .into_iter()
        .map(|g| GroupRef {
          id: g.id,
          name: g.name,
        })
        .collect(),
      paused_groups: r
        .paused_groups
        .into_iter()
        .map(|g| GroupRef {
          id: g.id,
          name: g.name,
        })
        .collect(),
      warnings: r
        .warnings
        .into_iter()
        .map(|w| ImportWarning {
          row_number: w.row_number,
          message: w.message,
        })
        .collect(),
    }
  }
}

pub async fn import_filters(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Json(body): Json<ImportFiltersRequest>,
) -> Result<Json<ApiResponse<ImportFiltersResponse>>, ApiAppError> {
  let rows = body
    .rows
    .into_iter()
    .map(|r| app::ImportFilterRowInput {
      ticker: r.ticker,
      company_name: r.company_name,
      group_name: r.group_name,
      notes: r.notes,
      enabled: r.enabled,
    })
    .collect();

  let result = app::import_filters_all(
    state.notify_group_repository.as_ref(),
    state.notify_filter_repository.as_ref(),
    state.unit_of_work.as_ref(),
    &state.import_settings,
    auth_user.user_id,
    rows,
    body.dry_run,
  )
  .await?;

  Ok(Json(ApiResponse::ok(ImportFiltersResponse::from(result))))
}

// ─── POST /api/groups/{id}/filters/import(グループ単位) ───
#[derive(Deserialize)]
pub struct ImportGroupFilterRow {
  pub ticker: String,
  pub company_name: String,
  pub notes: Option<String>,
  pub enabled: Option<bool>,
}

#[derive(Deserialize)]
pub struct ImportGroupFiltersRequest {
  pub rows: Vec<ImportGroupFilterRow>,
  #[serde(default)]
  pub dry_run: bool,
}

pub async fn import_group_filters(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path(group_id): Path<GroupId>,
  Json(body): Json<ImportGroupFiltersRequest>,
) -> Result<Json<ApiResponse<ImportFiltersResponse>>, ApiAppError> {
  let rows = body
    .rows
    .into_iter()
    .map(|r| app::ImportGroupFilterRowInput {
      ticker: r.ticker,
      company_name: r.company_name,
      notes: r.notes,
      enabled: r.enabled,
    })
    .collect();

  let result = app::import_filters_for_group(
    state.notify_group_repository.as_ref(),
    state.notify_filter_repository.as_ref(),
    &state.import_settings,
    auth_user.user_id,
    group_id,
    rows,
    body.dry_run,
  )
  .await?;

  Ok(Json(ApiResponse::ok(ImportFiltersResponse::from(result))))
}

// ─── GET /api/filters/export, /api/groups/{id}/filters/export ───
#[derive(Deserialize)]
pub struct ExportFiltersQuery {
  pub format: ExportFormat,
}

fn xlsx_response(bytes: Vec<u8>, filename: &str) -> impl IntoResponse {
  (
    [
      (
        header::CONTENT_TYPE,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
      ),
      (
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{filename}\""),
      ),
    ],
    Bytes::from(bytes),
  )
}

pub async fn export_filters(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Query(query): Query<ExportFiltersQuery>,
) -> Result<impl IntoResponse, ApiAppError> {
  let ExportFormat::Xlsx = query.format; // MVPではxlsxのみ(将来csv追加時はmatchに変更)

  let bytes = app::export_filters_all(
    state.notify_group_repository.as_ref(),
    state.notify_filter_repository.as_ref(),
    auth_user.user_id,
  )
  .await?;

  Ok(xlsx_response(bytes, "filters.xlsx"))
}

pub async fn export_group_filters(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path(group_id): Path<GroupId>,
  Query(query): Query<ExportFiltersQuery>,
) -> Result<impl IntoResponse, ApiAppError> {
  let ExportFormat::Xlsx = query.format;

  let bytes = app::export_filters_for_group(
    state.notify_group_repository.as_ref(),
    state.notify_filter_repository.as_ref(),
    auth_user.user_id,
    group_id,
  )
  .await?;

  Ok(xlsx_response(bytes, "group_filters.xlsx"))
}
