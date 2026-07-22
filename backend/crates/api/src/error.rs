/*
backend/crates/api/src/error.rs
apiクレートのエラー型の定義
*/

// 外部クレート
use axum::Json;
use axum::response::{IntoResponse, Response};

// 内部ライブラリ
/// ハンドラ内で発生したエラーをAPIエンベロープ形式のレスポンスへ変換する。
pub struct ApiAppError(pub ApiErrorCode, pub String);

// 自クレート
use crate::response::{ApiErrorCode, ApiResponse};

impl IntoResponse for ApiAppError {
  fn into_response(self) -> Response {
    let status = self.0.status_code();
    (status, Json(ApiResponse::<()>::err(self.0, self.1))).into_response()
  }
}

impl From<app::AppError> for ApiAppError {
  fn from(e: app::AppError) -> Self {
    match &e {
      app::AppError::UsernameAlreadyExists => {
        ApiAppError(ApiErrorCode::AlreadyExists, e.to_string())
      }
      app::AppError::InvalidInput(_) => ApiAppError(ApiErrorCode::InvalidRequest, e.to_string()),
      app::AppError::InvalidCredentials => ApiAppError(
        ApiErrorCode::Unauthorized,
        "ユーザ名またはパスワードが正しくありません".to_string(),
      ),
      app::AppError::UserDisabled => ApiAppError(
        ApiErrorCode::Forbidden,
        "アカウントが無効化されています".to_string(),
      ),
      app::AppError::SessionInvalid => ApiAppError(
        ApiErrorCode::Unauthorized,
        "セッションが無効です".to_string(),
      ),
      app::AppError::TokenError => ApiAppError(
        ApiErrorCode::InternalError,
        "トークンの処理に失敗しました".to_string(),
      ),
      app::AppError::NotFound => {
        ApiAppError(ApiErrorCode::NotFound, "対象が見つかりません".to_string())
      }
      app::AppError::Forbidden => ApiAppError(
        ApiErrorCode::Forbidden,
        "この操作を行う権限がありません".to_string(),
      ),
      app::AppError::CryptoError => ApiAppError(
        ApiErrorCode::InternalError,
        "設定の処理に失敗しました".to_string(),
      ),
      app::AppError::Repository(_) => ApiAppError(
        ApiErrorCode::InternalError,
        "内部エラーが発生しました".to_string(),
      ),
      app::AppError::ImportEmpty => ApiAppError(
        ApiErrorCode::ImportEmpty,
        "インポート対象の行が1件もありません".to_string(),
      ),
      // CLI専用エラー：本来ここには到達しないはず。バグとして検知する。
      app::AppError::ScraperError(_) | app::AppError::MonitorNotHealthy => {
        tracing::error!(error = ?e, "CLI専用エラーがAPI層に到達しました");
        ApiAppError(
          ApiErrorCode::InternalError,
          "内部エラーが発生しました".to_string(),
        )
      }
    }
  }
}
