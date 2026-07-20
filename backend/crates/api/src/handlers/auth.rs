/*
backend/crates/api/src/handlers/auth.rs
認証系のハンドラ
*/
use axum::Json;
use axum::extract::State;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::cookie::{
  ACCESS_TOKEN_COOKIE, REFRESH_TOKEN_COOKIE, build_access_token_cookie, build_refresh_token_cookie,
  build_removal_cookie,
};
use crate::error::ApiAppError;
use crate::extractor::AuthUser;
use crate::response::{ApiErrorCode, ApiResponse};
use crate::state::AppState;

// ─── POST /api/auth/login ───
#[derive(Deserialize)]
pub struct LoginRequest {
  pub username: String,
  pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
  pub username: String,
}

pub async fn login(
  State(state): State<AppState>,
  jar: CookieJar,
  Json(body): Json<LoginRequest>,
) -> Result<(CookieJar, Json<ApiResponse<LoginResponse>>), ApiAppError> {
  let output = app::login(
    state.user_repository.as_ref(),
    state.refresh_token_repository.as_ref(),
    &body.username,
    &body.password,
    None, // user_agent: 必要になればTypedHeader<UserAgent>を引数に追加して渡す
    &state.jwt_secret,
    state.access_token_ttl_minutes,
    state.refresh_token_ttl_days,
  )
  .await?;

  let jar = jar
    .add(build_access_token_cookie(
      output.access_token,
      state.cookie_secure,
      state.access_token_ttl_minutes,
    ))
    .add(build_refresh_token_cookie(
      output.refresh_token_plain,
      state.cookie_secure,
      state.refresh_token_ttl_days,
    ));

  Ok((
    jar,
    Json(ApiResponse::ok(LoginResponse {
      username: output.username,
    })),
  ))
}

// ─── POST /api/auth/refresh ───
// リクエストボディなし。レスポンスは ApiResponse::ok(()) (data: null)
pub async fn refresh(
  State(state): State<AppState>,
  jar: CookieJar,
) -> Result<(CookieJar, Json<ApiResponse<()>>), ApiAppError> {
  let refresh_token_plain = jar
    .get(REFRESH_TOKEN_COOKIE)
    .map(|c| c.value().to_string())
    .ok_or_else(|| {
      ApiAppError(
        ApiErrorCode::Unauthorized,
        "セッションが無効です".to_string(),
      )
    })?;

  let access_token = app::refresh(
    state.refresh_token_repository.as_ref(),
    state.user_repository.as_ref(),
    &refresh_token_plain,
    &state.jwt_secret,
    state.access_token_ttl_minutes,
  )
  .await?;

  let jar = jar.add(build_access_token_cookie(
    access_token,
    state.cookie_secure,
    state.access_token_ttl_minutes,
  ));

  Ok((jar, Json(ApiResponse::ok(()))))
}

// ─── POST /api/auth/logout ───
// リクエストボディなし。レスポンスは ApiResponse::ok(()) (data: null)
pub async fn logout(
  State(state): State<AppState>,
  jar: CookieJar,
) -> Result<(CookieJar, Json<ApiResponse<()>>), ApiAppError> {
  if let Some(cookie) = jar.get(REFRESH_TOKEN_COOKIE) {
    app::logout(state.refresh_token_repository.as_ref(), cookie.value()).await?;
  }

  let jar = jar
    .add(build_removal_cookie(ACCESS_TOKEN_COOKIE, "/api"))
    .add(build_removal_cookie(
      REFRESH_TOKEN_COOKIE,
      "/api/auth/refresh",
    ));

  Ok((jar, Json(ApiResponse::ok(()))))
}

// ─── GET /api/auth/me ───
#[derive(Serialize)]
pub struct MeResponse {
  pub username: String,
}

pub async fn me(
  State(state): State<AppState>,
  auth_user: AuthUser,
) -> Result<Json<ApiResponse<MeResponse>>, ApiAppError> {
  let user = state
    .user_repository
    .find_by_id(auth_user.user_id)
    .await
    .map_err(app::AppError::from)?
    .ok_or_else(|| ApiAppError(ApiErrorCode::Unauthorized, "認証が必要です".to_string()))?;

  Ok(Json(ApiResponse::ok(MeResponse {
    username: user.username,
  })))
}
