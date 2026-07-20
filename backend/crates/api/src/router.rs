/*
backend/crates/api/src/router.rs
ルータ定義
*/

// 外部ライブラリ
use axum::{
  Router,
  routing::{get, post},
};

// 自クレート
use crate::handlers::{auth, health};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
  Router::new()
    .route("/api/health", get(health::health))
    .route("/api/auth/login", post(auth::login))
    .route("/api/auth/refresh", post(auth::refresh))
    .route("/api/auth/logout", post(auth::logout))
    .route("/api/auth/me", get(auth::me))
    .with_state(state)
}
