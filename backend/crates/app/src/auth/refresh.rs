/*
backend/crates/app/src/auth/refresh.rs
リフレッシュのユースケース
*/

// 内部ライブラリ
use auth::issue_access_token;
use repository::{RefreshTokenRepository, UserRepository};

// 外部クレート
use crate::AppError;

pub async fn refresh(
  refresh_token_repo: &dyn RefreshTokenRepository,
  user_repo: &dyn UserRepository,
  refresh_token_plain: &str,
  jwt_secret: &str,
  access_token_ttl_minutes: i64,
) -> Result<String, AppError> {
  let token_hash = auth::hash_refresh_token(refresh_token_plain);

  let token = refresh_token_repo
    .find_by_token_hash(&token_hash)
    .await?
    .ok_or(AppError::SessionInvalid)?;

  if token.is_revoked() || token.is_expired_at(chrono::Utc::now()) {
    return Err(AppError::SessionInvalid);
  }

  let user = user_repo
    .find_by_id(token.user_id)
    .await?
    .ok_or(AppError::SessionInvalid)?;

  if user.is_disabled() {
    return Err(AppError::UserDisabled);
  }

  issue_access_token(user.id, user.role, access_token_ttl_minutes, jwt_secret)
    .map_err(|_| AppError::TokenError)
}
