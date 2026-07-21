/*
backend/crates/api/src/handlers/group.rs
グループ系のハンドラ
*/

// 外部クレート
use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use identity::GroupId;
use subscription::NotifyMedium;

// 自クレート
use crate::error::ApiAppError;
use crate::extractor::AuthUser;
use crate::handlers::common::GroupResponse;
use crate::response::ApiResponse;
use crate::state::AppState;

// ─── GET /api/groups ───
pub async fn list_groups(
  State(state): State<AppState>,
  auth_user: AuthUser,
) -> Result<Json<ApiResponse<Vec<GroupResponse>>>, ApiAppError> {
  let groups = app::list_groups(state.notify_group_repository.as_ref(), auth_user.user_id).await?;
  Ok(Json(ApiResponse::ok(
    groups.into_iter().map(GroupResponse::from).collect(),
  )))
}

// ─── POST /api/groups ───
#[derive(Deserialize)]
pub struct CreateGroupRequest {
  pub name: String,
  pub medium: NotifyMedium,
}

pub async fn create_group(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Json(body): Json<CreateGroupRequest>,
) -> Result<Json<ApiResponse<GroupResponse>>, ApiAppError> {
  let group = app::create_group(
    state.unit_of_work.as_ref(),
    auth_user.user_id,
    body.name,
    body.medium,
  )
  .await?;
  Ok(Json(ApiResponse::ok(GroupResponse::from(group))))
}

// ─── PUT /api/groups/{id} ───
#[derive(Deserialize)]
pub struct UpdateGroupRequest {
  pub name: String,
  pub medium: NotifyMedium,
}

pub async fn update_group(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path(group_id): Path<GroupId>,
  Json(body): Json<UpdateGroupRequest>,
) -> Result<Json<ApiResponse<GroupResponse>>, ApiAppError> {
  let group = app::update_group(
    state.notify_group_repository.as_ref(),
    auth_user.user_id,
    group_id,
    body.name,
    body.medium,
  )
  .await?;
  Ok(Json(ApiResponse::ok(GroupResponse::from(group))))
}

// ─── DELETE /api/groups/{id} ───
pub async fn delete_group(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path(group_id): Path<GroupId>,
) -> Result<Json<ApiResponse<()>>, ApiAppError> {
  app::delete_group(
    state.notify_group_repository.as_ref(),
    auth_user.user_id,
    group_id,
  )
  .await?;
  Ok(Json(ApiResponse::ok(())))
}

// ─── PATCH /api/groups/{id}/pause, /resume ───
pub async fn pause_group(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path(group_id): Path<GroupId>,
) -> Result<Json<ApiResponse<GroupResponse>>, ApiAppError> {
  let group = app::pause_group(
    state.notify_group_repository.as_ref(),
    auth_user.user_id,
    group_id,
  )
  .await?;
  Ok(Json(ApiResponse::ok(GroupResponse::from(group))))
}

pub async fn resume_group(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path(group_id): Path<GroupId>,
) -> Result<Json<ApiResponse<GroupResponse>>, ApiAppError> {
  let group = app::resume_group(
    state.notify_group_repository.as_ref(),
    auth_user.user_id,
    group_id,
  )
  .await?;
  Ok(Json(ApiResponse::ok(GroupResponse::from(group))))
}

// ─── GET/PUT /api/groups/{id}/config ───
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "medium", rename_all = "camelCase")]
pub enum GroupConfigDto {
  #[serde(rename_all = "camelCase")]
  Discord {
    webhook_url: Option<String>,
    embed_color: Option<String>,
    mention_enabled: bool,
    mention_targets: Vec<String>,
  },
  #[serde(rename_all = "camelCase")]
  Slack {
    webhook_url: Option<String>,
    mention_enabled: bool,
    mention_targets: Vec<String>,
  },
}

impl From<app::GroupConfigData> for GroupConfigDto {
  fn from(data: app::GroupConfigData) -> Self {
    match data {
      app::GroupConfigData::Discord {
        webhook_url,
        embed_color,
        mention_enabled,
        mention_targets,
      } => GroupConfigDto::Discord {
        webhook_url,
        embed_color,
        mention_enabled,
        mention_targets,
      },
      app::GroupConfigData::Slack {
        webhook_url,
        mention_enabled,
        mention_targets,
      } => GroupConfigDto::Slack {
        webhook_url,
        mention_enabled,
        mention_targets,
      },
    }
  }
}

impl From<GroupConfigDto> for app::GroupConfigData {
  fn from(dto: GroupConfigDto) -> Self {
    match dto {
      GroupConfigDto::Discord {
        webhook_url,
        embed_color,
        mention_enabled,
        mention_targets,
      } => app::GroupConfigData::Discord {
        webhook_url,
        embed_color,
        mention_enabled,
        mention_targets,
      },
      GroupConfigDto::Slack {
        webhook_url,
        mention_enabled,
        mention_targets,
      } => app::GroupConfigData::Slack {
        webhook_url,
        mention_enabled,
        mention_targets,
      },
    }
  }
}

pub async fn get_group_config(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path(group_id): Path<GroupId>,
) -> Result<Json<ApiResponse<GroupConfigDto>>, ApiAppError> {
  let data = app::get_group_config(
    state.notify_group_repository.as_ref(),
    state.notify_discord_config_repository.as_ref(),
    state.notify_slack_config_repository.as_ref(),
    auth_user.user_id,
    group_id,
    &state.webhook_enc_key,
  )
  .await?;

  Ok(Json(ApiResponse::ok(GroupConfigDto::from(data))))
}

pub async fn put_group_config(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path(group_id): Path<GroupId>,
  Json(body): Json<GroupConfigDto>,
) -> Result<Json<ApiResponse<()>>, ApiAppError> {
  app::put_group_config(
    state.notify_group_repository.as_ref(),
    state.notify_discord_config_repository.as_ref(),
    state.notify_slack_config_repository.as_ref(),
    auth_user.user_id,
    group_id,
    body.into(),
    &state.webhook_enc_key,
  )
  .await?;

  Ok(Json(ApiResponse::ok(())))
}

// ─── POST /api/groups/{id}/config/test-send ───
#[derive(Deserialize)]
pub struct TestSendRequest {
  pub ticker: Option<String>,
  pub company_name: Option<String>,
  pub title: Option<String>,
  pub evaluation: Option<earnings::EarningsEvaluation>,
  pub embed_color: Option<String>,
  pub webhook_url: Option<String>,
  pub mention_targets: Option<Vec<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestSendResponse {
  pub success: bool,
  pub failure_reason: Option<String>,
}

pub async fn test_send(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Path(group_id): Path<GroupId>,
  Json(body): Json<TestSendRequest>,
) -> Result<Json<ApiResponse<TestSendResponse>>, ApiAppError> {
  let output = app::test_send(
    state.notify_group_repository.as_ref(),
    state.notify_discord_config_repository.as_ref(),
    auth_user.user_id,
    group_id,
    app::TestSendInput {
      ticker: body.ticker,
      company_name: body.company_name,
      title: body.title,
      evaluation: body.evaluation,
      embed_color: body.embed_color,
      webhook_url: body.webhook_url,
      mention_targets: body.mention_targets,
    },
    &state.webhook_enc_key,
  )
  .await?;

  Ok(Json(ApiResponse::ok(TestSendResponse {
    success: output.success,
    failure_reason: output.failure_reason,
  })))
}

// ─── PUT /api/groups/bulk-destination ───
#[derive(Deserialize)]
pub struct BulkDestinationRequest {
  pub group_ids: Vec<GroupId>,
  pub config: GroupConfigDto,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkDestinationResponse {
  pub updated_count: u32,
}

pub async fn bulk_destination(
  State(state): State<AppState>,
  auth_user: AuthUser,
  Json(body): Json<BulkDestinationRequest>,
) -> Result<Json<ApiResponse<BulkDestinationResponse>>, ApiAppError> {
  let updated_count = app::bulk_destination(
    state.notify_group_repository.as_ref(),
    state.notify_discord_config_repository.as_ref(),
    state.notify_slack_config_repository.as_ref(),
    auth_user.user_id,
    body.group_ids,
    body.config.into(),
    &state.webhook_enc_key,
  )
  .await?;

  Ok(Json(ApiResponse::ok(BulkDestinationResponse {
    updated_count,
  })))
}
