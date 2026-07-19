/*
backend/crates/infra/src/postgres/notify_filter_repository.rs
通知フィルタテーブルのPostgres実装
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// 内部ライブラリ
use identity::{FilterId, GroupId};
use repository::{NotifyFilterRepository, RepositoryError};
use subscription::NotifyFilter;

// 自クレート
use crate::error_mapping::{map_conflict_error, map_error};

pub struct PgNotifyFilterRepository {
  pool: PgPool,
}

impl PgNotifyFilterRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

struct NotifyFilterRow {
  id: Uuid,
  group_id: Uuid,
  ticker: String,
  company_name: String,
  notes: Option<String>,
  enabled: bool,
  created_at: chrono::DateTime<chrono::Utc>,
}

impl From<NotifyFilterRow> for NotifyFilter {
  fn from(row: NotifyFilterRow) -> Self {
    NotifyFilter {
      id: FilterId::from_uuid(row.id),
      group_id: GroupId::from_uuid(row.group_id),
      ticker: row.ticker,
      company_name: row.company_name,
      notes: row.notes,
      enabled: row.enabled,
      created_at: row.created_at,
    }
  }
}

#[async_trait]
impl NotifyFilterRepository for PgNotifyFilterRepository {
  async fn find_by_id(&self, id: FilterId) -> Result<Option<NotifyFilter>, RepositoryError> {
    let row = sqlx::query_as!(
      NotifyFilterRow,
      r#"
      SELECT id, group_id, ticker, company_name, notes, enabled, created_at
      FROM notify_filters WHERE id = $1
      "#,
      id.as_uuid()
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(row.map(NotifyFilter::from))
  }

  async fn list_by_group_id(
    &self,
    group_id: GroupId,
  ) -> Result<Vec<NotifyFilter>, RepositoryError> {
    let rows = sqlx::query_as!(
      NotifyFilterRow,
      r#"
      SELECT id, group_id, ticker, company_name, notes, enabled, created_at
      FROM notify_filters WHERE group_id = $1 ORDER BY created_at ASC
      "#,
      group_id.as_uuid()
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(rows.into_iter().map(NotifyFilter::from).collect())
  }

  async fn insert(&self, filter: &NotifyFilter) -> Result<(), RepositoryError> {
    sqlx::query!(
      r#"
      INSERT INTO notify_filters (id, group_id, ticker, company_name, notes, enabled, created_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7)
      "#,
      filter.id.as_uuid(),
      filter.group_id.as_uuid(),
      filter.ticker,
      filter.company_name,
      filter.notes,
      filter.enabled,
      filter.created_at
    )
    .execute(&self.pool)
    .await
    .map_err(map_conflict_error)?;

    Ok(())
  }

  async fn update(&self, filter: &NotifyFilter) -> Result<(), RepositoryError> {
    sqlx::query!(
      r#"
      UPDATE notify_filters
      SET ticker = $2, company_name = $3, notes = $4, enabled = $5
      WHERE id = $1
      "#,
      filter.id.as_uuid(),
      filter.ticker,
      filter.company_name,
      filter.notes,
      filter.enabled
    )
    .execute(&self.pool)
    .await
    .map_err(map_error)?;

    Ok(())
  }

  async fn delete(&self, id: FilterId) -> Result<(), RepositoryError> {
    sqlx::query!("DELETE FROM notify_filters WHERE id = $1", id.as_uuid())
      .execute(&self.pool)
      .await
      .map_err(map_error)?;

    Ok(())
  }

  async fn replace_all_for_group(
    &self,
    group_id: GroupId,
    filters: &[NotifyFilter],
  ) -> Result<(), RepositoryError> {
    // CSVインポートの差分反映(00-overview.md 4章原則4)。DELETE + 一括INSERTを1トランザクションで行う。
    let mut tx = self.pool.begin().await.map_err(map_error)?;

    sqlx::query!(
      "DELETE FROM notify_filters WHERE group_id = $1",
      group_id.as_uuid()
    )
    .execute(&mut *tx)
    .await
    .map_err(map_error)?;

    for filter in filters {
      sqlx::query!(
        r#"INSERT INTO notify_filters (id, group_id, ticker, company_name, notes, enabled, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        filter.id.as_uuid(),
        filter.group_id.as_uuid(),
        filter.ticker,
        filter.company_name,
        filter.notes,
        filter.enabled,
        filter.created_at
      )
      .execute(&mut *tx)
      .await
      .map_err(map_conflict_error)?;
    }

    tx.commit().await.map_err(map_error)?;

    Ok(())
  }
}
