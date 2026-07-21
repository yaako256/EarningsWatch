/*
backend/crates/api/src/router.rs
ルータ定義
*/

// 外部ライブラリ
use axum::{
  Router,
  routing::{get, patch, post, put},
};

// 自クレート
use crate::handlers::{auth, filter, group, health};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
  Router::new()
    // ヘルスチェック
    .route("/api/health", get(health::health))
    // 認証系
    .route("/api/auth/login", post(auth::login))
    .route("/api/auth/refresh", post(auth::refresh))
    .route("/api/auth/logout", post(auth::logout))
    .route("/api/auth/me", get(auth::me))
    // グループ系
    .route(
      "/api/groups",
      get(group::list_groups).post(group::create_group),
    )
    .route("/api/groups/bulk-destination", put(group::bulk_destination))
    .route(
      "/api/groups/:id",
      put(group::update_group).delete(group::delete_group),
    )
    .route("/api/groups/:id/pause", patch(group::pause_group))
    .route("/api/groups/:id/resume", patch(group::resume_group))
    .route(
      "/api/groups/:id/config",
      get(group::get_group_config).put(group::put_group_config),
    )
    .route("/api/groups/:id/config/test-send", post(group::test_send))
    // グループごとのフィルタ系
    .route(
      "/api/groups/:id/filters",
      get(filter::list_filters).post(filter::create_filter),
    )
    .route(
      "/api/groups/:id/filters/:filter_id",
      put(filter::update_filter).delete(filter::delete_filter),
    )
    .route(
      "/api/groups/:id/filters/:filter_id/enable",
      patch(filter::enable_filter),
    )
    .route(
      "/api/groups/:id/filters/:filter_id/disable",
      patch(filter::disable_filter),
    )
    // フィルタの一括〇〇系
    .route("/api/filters/bulk-enable", post(filter::bulk_enable))
    .route("/api/filters/bulk-disable", post(filter::bulk_disable))
    .route("/api/filters/bulk-delete", post(filter::bulk_delete))
    .with_state(state)
}
