/*
backend/crates/api/src/handlers/admin.rs
管理者機能のユースケース
*/

// 外部クレート
use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use auth::Role;
use identity::UserId;
use logging::{LogLevel, LogProcess};
use subscription::NotifyMedium;

// 自クレート
use crate::error::ApiAppError;
use crate::extractor::AdminUser;
use crate::response::{ApiResponse, Page};
use crate::state::AppState;

// ─── GET /api/admin/logs ───
#[derive(Deserialize)]
pub struct ListLogsQuery {
  pub from: Option<DateTime<Utc>>,
  pub to: Option<DateTime<Utc>>,
  pub level: Option<LogLevel>,
  pub process: Option<LogProcess>,
  pub page: u32,
  pub per_page: u32,
}

#[derive(Serialize)]
pub struct LogResponse {
  pub id: i64,
  pub timestamp: DateTime<Utc>,
  pub level: LogLevel,
  pub process: LogProcess,
  pub target: String,
  pub message: Option<String>,
  pub fields: serde_json::Value,
}

impl From<logging::LogEntry> for LogResponse {
  fn from(e: logging::LogEntry) -> Self {
    Self {
      id: e.id,
      timestamp: e.timestamp,
      level: e.level,
      process: e.process,
      target: e.target,
      message: e.message,
      fields: e.fields,
    }
  }
}

pub async fn list_logs(
  State(state): State<AppState>,
  _admin: AdminUser,
  Query(query): Query<ListLogsQuery>,
) -> Result<Json<ApiResponse<Page<LogResponse>>>, ApiAppError> {
  let (entries, total_count) = app::list_logs(
    state.log_repository.as_ref(),
    query.from,
    query.to,
    query.level,
    query.process,
    query.page,
    query.per_page,
  )
  .await?;

  let total_pages = ((total_count as f64) / (query.per_page as f64)).ceil() as u32;

  Ok(Json(ApiResponse::ok(Page {
    items: entries.into_iter().map(LogResponse::from).collect(),
    page: query.page,
    per_page: query.per_page,
    total_count,
    total_pages,
  })))
}

// ─── GET /api/admin/users ───
#[derive(Deserialize)]
pub struct ListUsersQuery {
  pub page: u32,
  pub per_page: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserResponse {
  pub id: UserId,
  pub username: String,
  pub role: Role,
  pub created_at: DateTime<Utc>,
  pub disabled_at: Option<DateTime<Utc>>,
}

impl From<auth::User> for AdminUserResponse {
  fn from(u: auth::User) -> Self {
    Self {
      id: u.id,
      username: u.username,
      role: u.role,
      created_at: u.created_at,
      disabled_at: u.disabled_at,
    }
  }
}

pub async fn list_users(
  State(state): State<AppState>,
  _admin: AdminUser,
  Query(query): Query<ListUsersQuery>,
) -> Result<Json<ApiResponse<Page<AdminUserResponse>>>, ApiAppError> {
  let (users, total_count) =
    app::list_users(state.user_repository.as_ref(), query.page, query.per_page).await?;
  let total_pages = ((total_count as f64) / (query.per_page as f64)).ceil() as u32;

  Ok(Json(ApiResponse::ok(Page {
    items: users.into_iter().map(AdminUserResponse::from).collect(),
    page: query.page,
    per_page: query.per_page,
    total_count,
    total_pages,
  })))
}

// ─── POST /api/admin/users(仮ユーザ作成) ───
#[derive(Deserialize)]
pub struct CreateUserRequest {
  pub username: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserResponse {
  pub id: UserId,
  pub username: String,
  pub temporary_password: String,
}

pub async fn create_user(
  State(state): State<AppState>,
  _admin: AdminUser,
  Json(body): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<CreateUserResponse>>, ApiAppError> {
  let output = app::create_temp_user(state.user_repository.as_ref(), body.username).await?;

  Ok(Json(ApiResponse::ok(CreateUserResponse {
    id: output.user.id,
    username: output.user.username,
    temporary_password: output.temporary_password,
  })))
}

// ─── POST /api/admin/users/{id}/disable ───
pub async fn disable_user(
  State(state): State<AppState>,
  _admin: AdminUser,
  Path(user_id): Path<UserId>,
) -> Result<Json<ApiResponse<()>>, ApiAppError> {
  app::disable_user(state.user_repository.as_ref(), user_id).await?;
  Ok(Json(ApiResponse::ok(())))
}

// ─── GET /api/admin/users/{id}/summary ───
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSummaryResponse {
  pub group_count: u32,
  pub filter_count: u32,
  pub discord_group_count: u32,
  pub slack_group_count: u32,
}

pub async fn user_summary(
  State(state): State<AppState>,
  _admin: AdminUser,
  Path(user_id): Path<UserId>,
) -> Result<Json<ApiResponse<UserSummaryResponse>>, ApiAppError> {
  let data = app::user_summary(
    state.notify_group_repository.as_ref(),
    state.notify_filter_repository.as_ref(),
    user_id,
  )
  .await?;

  Ok(Json(ApiResponse::ok(UserSummaryResponse {
    group_count: data.group_count,
    filter_count: data.filter_count,
    discord_group_count: data.discord_group_count,
    slack_group_count: data.slack_group_count,
  })))
}

// ─── GET/PUT /api/admin/notify-config ───
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyConfigResponse {
  pub medium: NotifyMedium,
  pub webhook_url: Option<String>,
  pub mention_enabled: bool,
  pub mention_targets: Vec<String>,
}

#[derive(Deserialize)]
pub struct UpdateNotifyConfigRequest {
  pub medium: NotifyMedium,
  pub webhook_url: Option<String>,
  pub mention_enabled: bool,
  pub mention_targets: Vec<String>,
}

pub async fn get_notify_config(
  State(state): State<AppState>,
  _admin: AdminUser,
) -> Result<Json<ApiResponse<Option<NotifyConfigResponse>>>, ApiAppError> {
  let data = app::get_notify_config(
    state.system_notify_config_repository.as_ref(),
    &state.webhook_enc_key,
  )
  .await?;

  Ok(Json(ApiResponse::ok(data.map(|d| NotifyConfigResponse {
    medium: d.medium,
    webhook_url: d.webhook_url,
    mention_enabled: d.mention_enabled,
    mention_targets: d.mention_targets,
  }))))
}

pub async fn update_notify_config(
  State(state): State<AppState>,
  _admin: AdminUser,
  Json(body): Json<UpdateNotifyConfigRequest>,
) -> Result<Json<ApiResponse<()>>, ApiAppError> {
  app::update_notify_config(
    state.system_notify_config_repository.as_ref(),
    app::NotifyConfigData {
      medium: body.medium,
      webhook_url: body.webhook_url,
      mention_enabled: body.mention_enabled,
      mention_targets: body.mention_targets,
    },
    &state.webhook_enc_key,
  )
  .await?;

  Ok(Json(ApiResponse::ok(())))
}

// ─── GET /api/admin/dashboard ───
#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SystemRunType {
  Monitor,
  Notify,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemRunDuration {
  pub run_type: SystemRunType,
  pub run_at: DateTime<Utc>,
  pub duration_ms: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminDashboardResponse {
  pub total_earnings_count: i64,
  pub notify_success_rate: Option<f64>,
  pub last_monitor_run_at: Option<DateTime<Utc>>,
  pub run_durations: Vec<SystemRunDuration>,
}

pub async fn admin_dashboard(
  State(state): State<AppState>,
  _admin: AdminUser,
) -> Result<Json<ApiResponse<AdminDashboardResponse>>, ApiAppError> {
  let data = app::admin_dashboard(
    state.earnings_repository.as_ref(),
    state.system_run_repository.as_ref(),
    state.dashboard_settings.admin_recent_runs_count,
  )
  .await?;

  // 本書4.7節: repository層は生文字列で返すため、ここでSystemRunTypeへ変換する
  let run_durations = data
    .run_durations
    .into_iter()
    .filter_map(|(run_type_str, run_at, duration_ms)| {
      let run_type = match run_type_str.as_str() {
        "monitor" => SystemRunType::Monitor,
        "notify" => SystemRunType::Notify,
        _ => return None, // 想定外の値は無視する(DB制約上通常は発生しない)
      };
      Some(SystemRunDuration {
        run_type,
        run_at,
        duration_ms,
      })
    })
    .collect();

  Ok(Json(ApiResponse::ok(AdminDashboardResponse {
    total_earnings_count: data.total_earnings_count,
    notify_success_rate: data.notify_success_rate,
    last_monitor_run_at: data.last_monitor_run_at,
    run_durations,
  })))
}
