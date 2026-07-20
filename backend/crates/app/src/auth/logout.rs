/*
backend/crates/app/src/auth/logout.rs
ログアウトのユースケース
*/

// 内部ライブラリ
use repository::RefreshTokenRepository;

// 自クレート
use crate::AppError;

pub async fn logout(
  refresh_token_repo: &dyn RefreshTokenRepository,
  refresh_token_plain: &str,
) -> Result<(), AppError> {
  let token_hash = auth::hash_refresh_token(refresh_token_plain);

  if let Some(token) = refresh_token_repo.find_by_token_hash(&token_hash).await? {
    refresh_token_repo.revoke(token.id).await?;
  }

  Ok(())
}
