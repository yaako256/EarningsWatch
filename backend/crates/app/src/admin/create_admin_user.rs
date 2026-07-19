/*
backend/crates/app/src/admin/create_admin_user.rs
管理者ユーザを作成するユースケース
*/

// 内部ライブラリ
use auth::{Role, User, hash_password};
use identity::UserId;
use repository::UserRepository;

// 自クレート
use crate::error::{AppError, AppResult};

/// 管理者ユーザを作成するユースケース
/// ユーザ名の重複チェック・パスワードのハッシュ化・Userの組み立て・保存までを担う。
pub async fn create_admin_user(
  user_repo: &dyn UserRepository,
  username: String,
  password: &str,
) -> AppResult<User> {
  let username = username.trim().to_string();
  if username.is_empty() {
    return Err(AppError::InvalidInput(
      "ユーザ名を入力してください".to_string(),
    ));
  }
  if password.is_empty() {
    return Err(AppError::InvalidInput(
      "パスワードを入力してください".to_string(),
    ));
  }

  if user_repo.find_by_username(&username).await?.is_some() {
    return Err(AppError::UsernameAlreadyExists);
  }

  let password_hash = hash_password(password).map_err(|e| AppError::InvalidInput(e.to_string()))?;

  let now = chrono::Utc::now();

  let user = User {
    id: UserId::new(),
    username,
    password_hash,
    role: Role::Admin,
    created_at: now,
    updated_at: now,
    disabled_at: None,
  };

  user_repo.insert(&user).await?;

  Ok(user)
}
