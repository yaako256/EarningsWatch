/*
backend/crates/api/src/state.rs
アプリの共有状態を定義
*/

// 外部クレート
use std::sync::Arc;

// 内部ライブラリ
use repository::{RefreshTokenRepository, UserRepository};

/// axumのRouterへ`.with_state()`で渡す共有状態
#[derive(Clone)]
pub struct AppState {
  pub user_repository: Arc<dyn UserRepository>,
  pub refresh_token_repository: Arc<dyn RefreshTokenRepository>,
  pub jwt_secret: String,
  pub access_token_ttl_minutes: i64,
  pub refresh_token_ttl_days: i64,
  pub cookie_secure: bool,
}
