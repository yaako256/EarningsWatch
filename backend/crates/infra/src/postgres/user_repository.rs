/*
backend/crates/infra/src/postgres/user_repository.rs
ユーザテーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// 内部ライブラリ
use auth::{Role, User};
use identity::UserId;
use repository::{RepositoryResult, UserRepository};

// 自クレート
use crate::error_mapping::{map_conflict_error, map_error};

pub struct PgUserRepository {
  pool: PgPool,
}

impl PgUserRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

struct UserRow {
  id: Uuid,
  username: String,
  password_hash: String,
  role: Role,
  created_at: chrono::DateTime<chrono::Utc>,
  updated_at: chrono::DateTime<chrono::Utc>,
  disabled_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<UserRow> for User {
  fn from(row: UserRow) -> Self {
    User {
      id: UserId::from_uuid(row.id),
      username: row.username,
      password_hash: row.password_hash,
      role: row.role,
      created_at: row.created_at,
      updated_at: row.updated_at,
      disabled_at: row.disabled_at,
    }
  }
}

#[async_trait]
impl UserRepository for PgUserRepository {
  async fn find_by_id(&self, id: UserId) -> RepositoryResult<Option<User>> {
    let row = sqlx::query_as!(
      UserRow,
      r#"
      SELECT id, username, password_hash, role as "role: Role",
              created_at, updated_at, disabled_at
      FROM users
      WHERE id = $1
      "#,
      id.as_uuid()
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(row.map(User::from))
  }

  async fn find_by_username(&self, username: &str) -> RepositoryResult<Option<User>> {
    let row = sqlx::query_as!(
      UserRow,
      r#"
      SELECT id, username, password_hash, role as "role: Role",
              created_at, updated_at, disabled_at
      FROM users
      WHERE username = $1
      "#,
      username
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(row.map(User::from))
  }

  async fn list(&self, page: u32, per_page: u32) -> RepositoryResult<(Vec<User>, i64)> {
    let limit = per_page as i64;
    let offset = page.saturating_sub(1) as i64 * limit;

    let rows = sqlx::query_as!(
      UserRow,
      r#"
      SELECT id, username, password_hash, role as "role: Role",
              created_at, updated_at, disabled_at
      FROM users
      ORDER BY created_at DESC
      LIMIT $1 OFFSET $2
      "#,
      limit,
      offset
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    let total_count = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM users"#)
      .fetch_one(&self.pool)
      .await
      .map_err(map_error)?;

    Ok((rows.into_iter().map(User::from).collect(), total_count))
  }

  async fn insert(&self, user: &User) -> RepositoryResult<()> {
    sqlx::query!(
      r#"
      INSERT INTO users (id, username, password_hash, role, created_at, updated_at, disabled_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7)
      "#,
      user.id.as_uuid(),
      user.username,
      user.password_hash,
      user.role as Role,
      user.created_at,
      user.updated_at,
      user.disabled_at
    )
    .execute(&self.pool)
    .await
    .map_err(map_conflict_error)?;

    Ok(())
  }

  async fn update(&self, user: &User) -> RepositoryResult<()> {
    sqlx::query!(
      r#"
      UPDATE users
      SET username = $2, password_hash = $3, role = $4, updated_at = $5, disabled_at = $6
      WHERE id = $1
      "#,
      user.id.as_uuid(),
      user.username,
      user.password_hash,
      user.role as Role,
      user.updated_at,
      user.disabled_at
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }
}
