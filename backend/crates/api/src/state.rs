/*
backend/crates/api/src/state.rs
アプリのステートを定義
*/

/// axumのRouterへ`.with_state()`で渡す共有状態。
/// Phase 2時点では空。Phase 5以降でRepository実装・DBプール等を追加していく。
#[derive(Clone)]
pub struct AppState {}
