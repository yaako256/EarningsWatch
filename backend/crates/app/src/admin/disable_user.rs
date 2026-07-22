/*
backend/crates/app/src/admin/disable_user.rs
ユーザを無効化(Ban)するユースケース
*/

// 内部ライブラリ
use identity::UserId;
use repository::UserRepository;

// 自クレート
use crate::AppError;

pub async fn disable_user(user_repo: &dyn UserRepository, user_id: UserId) -> Result<(), AppError> {
  let mut user = user_repo
    .find_by_id(user_id)
    .await?
    .ok_or(AppError::NotFound)?;
  user.disabled_at = Some(chrono::Utc::now());
  user.updated_at = chrono::Utc::now();
  user_repo.update(&user).await?;
  Ok(())
}
