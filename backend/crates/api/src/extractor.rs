/*
backend/crates/api/src/extractor.rs
JWT認証ミドルウェア
*/

// 外部クレート
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;

//内部ライブラリ
use auth::{Role, verify_access_token};
use identity::UserId;

// 自クレート
use crate::cookie::ACCESS_TOKEN_COOKIE;
use crate::error::ApiAppError;
use crate::response::ApiErrorCode;
use crate::state::AppState;

pub struct AuthUser {
  pub user_id: UserId,
  pub role: Role,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
  type Rejection = ApiAppError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let jar = CookieJar::from_headers(&parts.headers);
    let token = jar
      .get(ACCESS_TOKEN_COOKIE)
      .map(|c| c.value().to_string())
      .ok_or_else(|| ApiAppError(ApiErrorCode::Unauthorized, "認証が必要です".to_string()))?;

    let claims = verify_access_token(&token, &state.jwt_secret)
      .map_err(|_| ApiAppError(ApiErrorCode::Unauthorized, "認証が必要です".to_string()))?;

    let user_id = claims
      .sub
      .parse::<uuid::Uuid>()
      .map(UserId::from_uuid)
      .map_err(|_| ApiAppError(ApiErrorCode::Unauthorized, "認証が必要です".to_string()))?;

    Ok(AuthUser {
      user_id,
      role: claims.role,
    })
  }
}

/// 管理者権限が必要なエンドポイント用
pub struct AdminUser(pub AuthUser);

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
  type Rejection = ApiAppError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let user = AuthUser::from_request_parts(parts, state).await?;
    if !user.role.is_admin() {
      return Err(ApiAppError(
        ApiErrorCode::Forbidden,
        "管理者権限が必要です".to_string(),
      ));
    }
    Ok(AdminUser(user))
  }
}
