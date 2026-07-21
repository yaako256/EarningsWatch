/*
backend/crates/infra/src/postgres/notify_history_repository.rs
送信履歴テーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// 内部ライブラリ
use identity::GroupId;
use repository::{NotifyHistoryRepository, RepositoryResult};
use subscription::{NotifyHistoryEntry, NotifyStatus};

// 自クレート
use crate::error_mapping::map_error;

pub struct PgNotifyHistoryRepository {
  pool: PgPool,
}

impl PgNotifyHistoryRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

struct NotifyHistoryRow {
  id: i64,
  group_id: Option<Uuid>,
  fingerprint: String,
  sent_at: chrono::DateTime<chrono::Utc>,
  status: NotifyStatus,
}

impl From<NotifyHistoryRow> for NotifyHistoryEntry {
  fn from(row: NotifyHistoryRow) -> Self {
    NotifyHistoryEntry {
      id: row.id,
      group_id: row.group_id.map(GroupId::from_uuid),
      fingerprint: row.fingerprint,
      sent_at: row.sent_at,
      status: row.status,
    }
  }
}

#[async_trait]
impl NotifyHistoryRepository for PgNotifyHistoryRepository {
  async fn insert(&self, entry: &NotifyHistoryEntry) -> RepositoryResult<()> {
    sqlx::query!(
      r#"
      INSERT INTO notify_history (group_id, fingerprint, sent_at, status)
      VALUES ($1, $2, $3, $4)
      "#,
      entry.group_id.map(|g| *g.as_uuid()),
      entry.fingerprint,
      entry.sent_at,
      entry.status as NotifyStatus,
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }

  async fn list_by_group_id(
    &self,
    group_id: GroupId,
    page: u32,
    per_page: u32,
  ) -> RepositoryResult<(Vec<NotifyHistoryEntry>, i64)> {
    let limit = per_page as i64;
    let offset = page.saturating_sub(1) as i64 * limit;

    let rows = sqlx::query_as!(
      NotifyHistoryRow,
      r#"
      SELECT id, group_id, fingerprint, sent_at, status as "status: NotifyStatus"
      FROM notify_history WHERE group_id = $1
      ORDER BY sent_at DESC LIMIT $2 OFFSET $3
      "#,
      group_id.as_uuid(),
      limit,
      offset
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    let total_count = sqlx::query_scalar!(
      r#"SELECT COUNT(*) as "count!" FROM notify_history WHERE group_id = $1"#,
      group_id.as_uuid()
    )
    .fetch_one(&self.pool)
    .await
    .map_err(map_error)?;

    Ok((
      rows.into_iter().map(NotifyHistoryEntry::from).collect(),
      total_count,
    ))
  }

  async fn list_all(
    &self,
    page: u32,
    per_page: u32,
  ) -> RepositoryResult<(Vec<NotifyHistoryEntry>, i64)> {
    let limit = per_page as i64;
    let offset = page.saturating_sub(1) as i64 * limit;

    let rows = sqlx::query_as!(
      NotifyHistoryRow,
      r#"
      SELECT id, group_id, fingerprint, sent_at, status as "status: NotifyStatus"
      FROM notify_history
      ORDER BY sent_at DESC LIMIT $1 OFFSET $2
      "#,
      limit,
      offset
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    let total_count = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM notify_history"#)
      .fetch_one(&self.pool)
      .await
      .map_err(map_error)?;

    Ok((
      rows.into_iter().map(NotifyHistoryEntry::from).collect(),
      total_count,
    ))
  }

  async fn list_recent_by_user_since(
    &self,
    user_id: identity::UserId,
    status: NotifyStatus,
    since: chrono::DateTime<chrono::Utc>,
  ) -> RepositoryResult<Vec<NotifyHistoryEntry>> {
    let rows = sqlx::query_as!(
      NotifyHistoryRow,
      r#"
      SELECT h.id, h.group_id, h.fingerprint, h.sent_at, h.status as "status: NotifyStatus"
      FROM notify_history h
      JOIN notify_groups g ON g.id = h.group_id
      WHERE g.user_id = $1 AND h.status = $2 AND h.sent_at >= $3
      ORDER BY h.sent_at DESC
      "#,
      user_id.as_uuid(),
      status as NotifyStatus,
      since
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(rows.into_iter().map(NotifyHistoryEntry::from).collect())
  }

  async fn list_recent_by_user_top_n(
    &self,
    user_id: identity::UserId,
    status: NotifyStatus,
    limit: u32,
  ) -> RepositoryResult<Vec<NotifyHistoryEntry>> {
    let rows = sqlx::query_as!(
      NotifyHistoryRow,
      r#"
      SELECT h.id, h.group_id, h.fingerprint, h.sent_at, h.status as "status: NotifyStatus"
      FROM notify_history h
      JOIN notify_groups g ON g.id = h.group_id
      WHERE g.user_id = $1 AND h.status = $2
      ORDER BY h.sent_at DESC
      LIMIT $3
      "#,
      user_id.as_uuid(),
      status as NotifyStatus,
      limit as i64
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(rows.into_iter().map(NotifyHistoryEntry::from).collect())
  }
}
