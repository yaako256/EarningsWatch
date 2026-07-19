/*
backend/crates/infra/src/postgres/refresh_token_repository.rs
リフレッシュトークンテーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// 内部ライブラリ
use auth::RefreshToken;
use identity::{RefreshTokenId, UserId};
use repository::{RefreshTokenRepository, RepositoryError};

// 自クレート
use crate::error_mapping::{map_conflict_error, map_error};

pub struct PgRefreshTokenRepository {
  pool: PgPool,
}

impl PgRefreshTokenRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

struct RefreshTokenRow {
  id: Uuid,
  user_id: Uuid,
  token_hash: String,
  user_agent: Option<String>,
  expires_at: chrono::DateTime<chrono::Utc>,
  created_at: chrono::DateTime<chrono::Utc>,
  revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<RefreshTokenRow> for RefreshToken {
  fn from(row: RefreshTokenRow) -> Self {
    RefreshToken {
      id: RefreshTokenId::from_uuid(row.id),
      user_id: UserId::from_uuid(row.user_id),
      token_hash: row.token_hash,
      user_agent: row.user_agent,
      expires_at: row.expires_at,
      created_at: row.created_at,
      revoked_at: row.revoked_at,
    }
  }
}

#[async_trait]
impl RefreshTokenRepository for PgRefreshTokenRepository {
  async fn find_by_token_hash(
    &self,
    token_hash: &str,
  ) -> Result<Option<RefreshToken>, RepositoryError> {
    let row = sqlx::query_as!(
      RefreshTokenRow,
      r#"
      SELECT id, user_id, token_hash, user_agent, expires_at, created_at, revoked_at
      FROM refresh_tokens WHERE token_hash = $1
      "#,
      token_hash
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(row.map(RefreshToken::from))
  }

  async fn list_by_user_id(&self, user_id: UserId) -> Result<Vec<RefreshToken>, RepositoryError> {
    let rows = sqlx::query_as!(
      RefreshTokenRow,
      r#"
      SELECT id, user_id, token_hash, user_agent, expires_at, created_at, revoked_at
      FROM refresh_tokens WHERE user_id = $1 ORDER BY created_at DESC
      "#,
      user_id.as_uuid()
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(rows.into_iter().map(RefreshToken::from).collect())
  }

  async fn insert(&self, token: &RefreshToken) -> Result<(), RepositoryError> {
    sqlx::query!(
      r#"
      INSERT INTO refresh_tokens (id, user_id, token_hash, user_agent, expires_at, created_at, revoked_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
      token.id.as_uuid(),
      token.user_id.as_uuid(),
      token.token_hash,
      token.user_agent,
      token.expires_at,
      token.created_at,
      token.revoked_at
    )
    .execute(&self.pool)
    .await
    .map_err(map_conflict_error)?;

    Ok(())
  }

  async fn revoke(&self, id: RefreshTokenId) -> Result<(), RepositoryError> {
    sqlx::query!(
      "UPDATE refresh_tokens SET revoked_at = now() WHERE id = $1",
      id.as_uuid()
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }

  async fn revoke_all_for_user(&self, user_id: UserId) -> Result<(), RepositoryError> {
    sqlx::query!(
      "UPDATE refresh_tokens SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
      user_id.as_uuid()
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }
}
