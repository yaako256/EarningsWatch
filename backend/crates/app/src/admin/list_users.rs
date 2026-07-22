/*
backend/crates/app/src/admin/list_users.rs
ユーザリストのユースケース
*/

// 内部ライブラリ
use auth::User;
use repository::UserRepository;

// 自クレート
use crate::AppError;

pub async fn list_users(
  user_repo: &dyn UserRepository,
  page: u32,
  per_page: u32,
) -> Result<(Vec<User>, i64), AppError> {
  Ok(user_repo.list(page, per_page).await?)
}
