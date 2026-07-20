/*
backend/crates/api/src/cookie.rs
Cookie系の定義
*/

// 外部クレート
use axum_extra::extract::cookie::{Cookie, SameSite};
use time::Duration;

// coockie上での名前
pub const ACCESS_TOKEN_COOKIE: &str = "access_token";
pub const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

pub fn build_access_token_cookie(token: String, secure: bool, ttl_minutes: i64) -> Cookie<'static> {
  Cookie::build((ACCESS_TOKEN_COOKIE, token))
    .path("/api")
    .http_only(true)
    .secure(secure)
    .same_site(SameSite::Strict)
    .max_age(Duration::minutes(ttl_minutes))
    .build()
}

pub fn build_refresh_token_cookie(token: String, secure: bool, ttl_days: i64) -> Cookie<'static> {
  Cookie::build((REFRESH_TOKEN_COOKIE, token))
    .path("/api/auth/refresh")
    .http_only(true)
    .secure(secure)
    .same_site(SameSite::Strict)
    .max_age(Duration::days(ttl_days))
    .build()
}

/// ログアウト時、Cookieを即時失効させるための削除用Cookie。
pub fn build_removal_cookie(name: &'static str, path: &'static str) -> Cookie<'static> {
  Cookie::build((name, ""))
    .path(path)
    .http_only(true)
    .max_age(Duration::ZERO)
    .build()
}
