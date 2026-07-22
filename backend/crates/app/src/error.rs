/*
backend/crates/app/src/error.rs
appクレートのエラー型の定義
*/

// 外部クレート
// エラー型作成用
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("ユーザ名は既に使用されています")]
  UsernameAlreadyExists,
  #[error("入力が不正です: {0}")]
  InvalidInput(String),
  #[error("ユーザ名またはパスワードが正しくありません")]
  InvalidCredentials,
  #[error("アカウントが無効化されています")]
  UserDisabled,
  #[error("セッションが無効です")]
  SessionInvalid,
  #[error("トークンの処理に失敗しました")]
  TokenError,
  #[error("対象が見つかりません")]
  NotFound,
  #[error("権限がありません")]
  Forbidden, //(他ユーザのグループ操作等)
  #[error("設定の暗号化/復号に失敗しました")]
  CryptoError,
  #[error("インポート対象の行が1件もありません")]
  ImportEmpty,
  #[error(transparent)]
  Repository(#[from] repository::RepositoryError),
  #[error("スクレイピングに失敗しました: {0}")]
  ScraperError(String), // 追加
  #[error("monitorが正常に完了していません")]
  MonitorNotHealthy,
}
/// appクレートのリザルト
pub(crate) type AppResult<T> = Result<T, AppError>;
