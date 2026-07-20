/*
backend/crates/auth/src/login.rs
ログインのユースケース
*/

// 外部クレート
use chrono::{DateTime, Utc};

// 内部ライブラリ
use auth::{Role, issue_access_token, verify_password};
use identity::RefreshTokenId;
use repository::{RefreshTokenRepository, UserRepository};

// 自クレート
use crate::AppError;

pub struct LoginOutput {
  pub username: String,
  pub role: Role,
  pub access_token: String,
  pub refresh_token_plain: String,
  pub refresh_token_expires_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
pub async fn login(
  user_repo: &dyn UserRepository,
  refresh_token_repo: &dyn RefreshTokenRepository,
  username: &str,
  password: &str,
  user_agent: Option<String>,
  jwt_secret: &str,
  access_token_ttl_minutes: i64,
  refresh_token_ttl_days: i64,
) -> Result<LoginOutput, AppError> {
  let user = user_repo
    .find_by_username(username)
    .await?
    .ok_or(AppError::InvalidCredentials)?;

  if user.is_disabled() {
    return Err(AppError::UserDisabled);
  }

  let is_valid =
    verify_password(password, &user.password_hash).map_err(|_| AppError::InvalidCredentials)?;
  if !is_valid {
    return Err(AppError::InvalidCredentials);
  }

  let access_token = issue_access_token(user.id, user.role, access_token_ttl_minutes, jwt_secret)
    .map_err(|_| AppError::TokenError)?;

  let refresh_token_plain = auth::generate_refresh_token_plain();
  let now = Utc::now();
  let expires_at = now + chrono::Duration::days(refresh_token_ttl_days);

  let refresh_token = auth::RefreshToken {
    id: RefreshTokenId::new(),
    user_id: user.id,
    token_hash: auth::hash_refresh_token(&refresh_token_plain),
    user_agent,
    expires_at,
    created_at: now,
    revoked_at: None,
  };

  refresh_token_repo.insert(&refresh_token).await?;

  Ok(LoginOutput {
    username: user.username,
    role: user.role,
    access_token,
    refresh_token_plain,
    refresh_token_expires_at: expires_at,
  })
}
