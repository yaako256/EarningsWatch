/*
backend/crates/api/src/response.rs
レスポンス型を定義
エンベロープで返す
*/

// 外部クレート
use axum::http::StatusCode;
use serde::Serialize;

/// レスポンス型(エンベロープ)
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
  pub data: Option<T>,
  pub error: Option<ApiError>,
}

impl<T: Serialize> ApiResponse<T> {
  pub fn ok(data: T) -> Self {
    Self {
      data: Some(data),
      error: None,
    }
  }
}

impl ApiResponse<()> {
  pub fn err(code: ApiErrorCode, message: impl Into<String>) -> Self {
    Self {
      data: None,
      error: Some(ApiError {
        code,
        message: message.into(),
      }),
    }
  }
}

#[derive(Serialize)]
pub struct ApiError {
  pub code: ApiErrorCode,
  pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiErrorCode {
  Unauthorized,
  Forbidden,
  NotFound,
  AlreadyExists,
  InvalidRequest,
  NotifyConfigMissing,
  NotifySendFailed,
  NotifyRejected,
  ImportEmpty,
  InternalError,
}

impl ApiErrorCode {
  pub fn status_code(self) -> StatusCode {
    match self {
      Self::Unauthorized => StatusCode::UNAUTHORIZED,
      Self::Forbidden => StatusCode::FORBIDDEN,
      Self::NotFound => StatusCode::NOT_FOUND,
      Self::AlreadyExists => StatusCode::CONFLICT,
      Self::InvalidRequest => StatusCode::UNPROCESSABLE_ENTITY,
      Self::NotifyConfigMissing => StatusCode::UNPROCESSABLE_ENTITY,
      Self::NotifySendFailed => StatusCode::BAD_GATEWAY,
      Self::NotifyRejected => StatusCode::BAD_GATEWAY,
      Self::ImportEmpty => StatusCode::UNPROCESSABLE_ENTITY,
      Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T: Serialize> {
  pub items: Vec<T>,
  pub page: u32,
  pub per_page: u32,
  pub total_count: i64,
  pub total_pages: u32,
}
