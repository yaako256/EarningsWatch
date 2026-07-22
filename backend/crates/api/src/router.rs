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
use crate::handlers::*;
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
    // インポート/エクスポート
    .route("/api/filters/import", post(filter::import_filters))
    .route(
      "/api/groups/:id/filters/import",
      post(filter::import_group_filters),
    )
    .route("/api/filters/export", get(filter::export_filters))
    .route(
      "/api/groups/:id/filters/export",
      get(filter::export_group_filters),
    )
    .route("/api/earnings/export", get(earnings::export_earnings))
    // ダッシュボード
    .route("/api/dashboard", get(dashboard::get_dashboard))
    // 決算情報系
    .route("/api/earnings", get(earnings::list_earnings))
    .route("/api/earnings/summary", get(earnings::earnings_summary))
    // 送信キュー/履歴
    .route("/api/notify-queue", get(notify_queue::list_notify_queue))
    .route(
      "/api/notify-history",
      get(notify_history::list_notify_history),
    )
    // 管理者機能
    .route("/api/admin/logs", get(admin::list_logs))
    .route(
      "/api/admin/users",
      get(admin::list_users).post(admin::create_user),
    )
    .route("/api/admin/users/:id/disable", post(admin::disable_user))
    .route("/api/admin/users/:id/summary", get(admin::user_summary))
    .route(
      "/api/admin/notify-config",
      get(admin::get_notify_config).put(admin::update_notify_config),
    )
    .route("/api/admin/dashboard", get(admin::admin_dashboard))
    // ページ機能系
    .route("/api/pages", get(page::list_pages).post(page::create_page))
    .route(
      "/api/pages/:id",
      get(page::get_page)
        .put(page::update_page)
        .delete(page::delete_page),
    )
    .route(
      "/api/pages/:id/order",
      axum::routing::patch(page::update_page_order),
    )
    .with_state(state)
}
