/*
backend/crates/app/src/admin/create_temp_user.rs
仮ユーザを発行するユースケース
*/

// 内部ライブラリ
use auth::{Role, User, hash_password};
use identity::UserId;
use rand::Rng;
use repository::UserRepository;

// 自クレート
use crate::AppError;

// 紛らわしい文字(0/O、1/l/I)を除いた文字集合(これ天才だろ)
const PASSWORD_CHARS: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz23456789";
const TEMP_PASSWORD_LEN: usize = 12;

pub struct CreateTempUserOutput {
  pub user: User,
  pub temporary_password: String,
}

pub async fn create_temp_user(
  user_repo: &dyn UserRepository,
  username: String,
) -> Result<CreateTempUserOutput, AppError> {
  let username = username.trim().to_string();
  if username.is_empty() {
    return Err(AppError::InvalidInput(
      "ユーザ名を入力してください".to_string(),
    ));
  }

  if user_repo.find_by_username(&username).await?.is_some() {
    return Err(AppError::UsernameAlreadyExists);
  }

  let temporary_password = generate_temp_password();
  let password_hash = hash_password(&temporary_password).map_err(|_| AppError::CryptoError)?;

  let now = chrono::Utc::now();
  let user = User {
    id: UserId::new(),
    username,
    password_hash,
    role: Role::User, // 仮ユーザは一般ユーザ権限(管理者は別途create-admin CLIで作成)
    created_at: now,
    updated_at: now,
    disabled_at: None,
  };

  user_repo.insert(&user).await?;

  Ok(CreateTempUserOutput {
    user,
    temporary_password,
  })
}

fn generate_temp_password() -> String {
  let mut rng = rand::thread_rng();
  (0..TEMP_PASSWORD_LEN)
    .map(|_| {
      let idx = rng.gen_range(0..PASSWORD_CHARS.len());
      PASSWORD_CHARS[idx] as char
    })
    .collect()
}
