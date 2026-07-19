/*
backend/crates/api/src/handlers/health.rs
ヘルスチェックのハンドラを定義
*/

// 外部クレート
use axum::Json;
// シリアライズ用
use serde::Serialize;

// 自クレート
// レスポンス共通型
use crate::response::ApiResponse;

#[derive(Serialize)]
pub struct HealthData {
  pub status: &'static str,
}

// ヘルスチェック
pub async fn health() -> Json<ApiResponse<HealthData>> {
  Json(ApiResponse::ok(HealthData { status: "ok" }))
}
