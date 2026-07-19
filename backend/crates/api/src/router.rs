/*
backend/crates/api/src/router.rs
ルータ定義
*/

// 外部ライブラリ
use axum::{Router, routing::get};

// 自クレート
use crate::handlers::*;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
  Router::new()
    .route("/api/health", get(health::health))
    .with_state(state)
}
