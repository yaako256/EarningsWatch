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
use repository::{NotifyFilterRepository, RepositoryResult};
use subscription::NotifyFilter;

// 自クレート
use super::queries::notify_filter;
use crate::error_mapping::map_error;

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
  async fn find_by_id(&self, id: FilterId) -> RepositoryResult<Option<NotifyFilter>> {
    notify_filter::find_by_id(&self.pool, id).await
  }

  async fn list_by_group_id(&self, group_id: GroupId) -> RepositoryResult<Vec<NotifyFilter>> {
    notify_filter::list_by_group_id(&self.pool, group_id).await
  }

  async fn insert(&self, filter: &NotifyFilter) -> RepositoryResult<()> {
    notify_filter::insert(&self.pool, filter).await
  }

  async fn update(&self, filter: &NotifyFilter) -> RepositoryResult<()> {
    notify_filter::update(&self.pool, filter).await
  }

  async fn delete(&self, id: FilterId) -> RepositoryResult<()> {
    notify_filter::delete(&self.pool, id).await
  }

  async fn replace_all_for_group(
    &self,
    group_id: GroupId,
    filters: &[NotifyFilter],
  ) -> RepositoryResult<()> {
    // CSVインポートの差分反映(00-overview.md 4章原則4)。DELETE + 一括INSERTを1トランザクションで行う。
    let mut tx = self.pool.begin().await.map_err(map_error)?;

    notify_filter::replace_all_for_group(&mut tx, group_id, filters).await?;

    tx.commit().await.map_err(map_error)?;

    Ok(())
  }
}
