/*
backend/crates/infra/src/postgres/unit_of_work.rs
uowのPostgres実装
*/

/*
メモ
重要:
上記のNotifyGroupRepository for PgTxRepositories実装は、
紙面の都合上find_by_idのみtodo!()のまま示している。
実装時は6.2〜6.4節の
各SQL(
PgNotifyGroupRepository/PgNotifyDiscordConfigRepository/PgNotifyFilterRepository/
PgEarningsRepository/PgNotifyQueueRepository)
を、executorだけ&self.poolから&mut **tx(ロック取得後のトランザクション)
に差し替えて全メソッド分コピーする必要がある。
コードの重複が気になる場合は、
6章の各リポジトリの実装を「executorを引数に取るフリー関数」に切り出し、PgXxxRepository(プール版)と
PgTxRepositories(トランザクション版)の両方から呼び出す形にリファクタリングしてよい
(本書では実装直前レベルの型・シグネチャを優先し、リファクタリングの余地として残した)。

→ リファクタリングした。`queries/`に共通化関数を書き、その関数に委譲する形にして重複をなくした。
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::Mutex;

// 内部ライブラリ
use repository::{
  BoxFuture, EarningsRepository, NotifyDiscordConfigRepository, NotifyFilterRepository,
  NotifyGroupRepository, NotifyQueueRepository, RepositoryError, RepositoryScope, UnitOfWork,
};

// 自クレート
// クエリ共通化関数
use super::queries::{
  earnings_query, notify_discord_config, notify_filter, notify_group, notify_queue,
};

/// 1トランザクション内で動く全Repositoryを1つの構造体にまとめ、
/// tokio::sync::Mutexでトランザクションへの排他アクセスを行う
pub struct PgTxRepositories {
  tx: Mutex<Transaction<'static, Postgres>>,
}

impl PgTxRepositories {
  fn new(tx: Transaction<'static, Postgres>) -> Self {
    Self { tx: Mutex::new(tx) }
  }

  async fn into_inner(self) -> Transaction<'static, Postgres> {
    self.tx.into_inner()
  }
}

#[async_trait]
impl NotifyGroupRepository for PgTxRepositories {
  async fn find_by_id(
    &self,
    id: identity::GroupId,
  ) -> Result<Option<subscription::NotifyGroup>, RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_group::find_by_id(&mut **tx, id).await
  }

  async fn list_by_user_id(
    &self,
    user_id: identity::UserId,
  ) -> Result<Vec<subscription::NotifyGroup>, RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_group::list_by_user_id(&mut **tx, user_id).await
  }

  async fn list_all(&self) -> Result<Vec<subscription::NotifyGroup>, RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_group::list_all(&mut **tx).await
  }

  async fn insert(&self, group: &subscription::NotifyGroup) -> Result<(), RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_group::insert(&mut **tx, group).await
  }

  async fn update(&self, group: &subscription::NotifyGroup) -> Result<(), RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_group::update(&mut **tx, group).await
  }

  async fn delete(&self, id: identity::GroupId) -> Result<(), RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_group::delete(&mut **tx, id).await
  }
}

#[async_trait]
impl NotifyDiscordConfigRepository for PgTxRepositories {
  async fn find_by_group_id(
    &self,
    group_id: identity::GroupId,
  ) -> Result<Option<repository::NotifyDiscordConfigRow>, RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_discord_config::find_by_group_id(&mut **tx, group_id).await
  }

  async fn upsert(
    &self,
    group_id: identity::GroupId,
    row: &repository::NotifyDiscordConfigRow,
  ) -> Result<(), RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_discord_config::upsert(&mut **tx, group_id, row).await
  }
}

#[async_trait]
impl NotifyFilterRepository for PgTxRepositories {
  async fn find_by_id(
    &self,
    id: identity::FilterId,
  ) -> Result<Option<subscription::NotifyFilter>, RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_filter::find_by_id(&mut **tx, id).await
  }

  async fn list_by_group_id(
    &self,
    group_id: identity::GroupId,
  ) -> Result<Vec<subscription::NotifyFilter>, RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_filter::list_by_group_id(&mut **tx, group_id).await
  }

  async fn insert(&self, filter: &subscription::NotifyFilter) -> Result<(), RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_filter::insert(&mut **tx, filter).await
  }

  async fn update(&self, filter: &subscription::NotifyFilter) -> Result<(), RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_filter::update(&mut **tx, filter).await
  }

  async fn delete(&self, id: identity::FilterId) -> Result<(), RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_filter::delete(&mut **tx, id).await
  }

  async fn replace_all_for_group(
    &self,
    group_id: identity::GroupId,
    filters: &[subscription::NotifyFilter],
  ) -> Result<(), RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_filter::replace_all_for_group(&mut *tx, group_id, filters).await
  }
}

#[async_trait]
impl EarningsRepository for PgTxRepositories {
  async fn find_by_fingerprint(
    &self,
    fingerprint: &str,
  ) -> Result<Option<earnings::EarningsRecord>, RepositoryError> {
    let mut tx = self.tx.lock().await;
    earnings_query::find_by_fingerprint(&mut **tx, fingerprint).await
  }

  async fn list_recent_fingerprints(&self, limit: u32) -> Result<Vec<String>, RepositoryError> {
    let mut tx = self.tx.lock().await;
    earnings_query::list_recent_fingerprints(&mut **tx, limit).await
  }

  async fn list(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<earnings::EarningsRecord>, i64), RepositoryError> {
    let mut tx = self.tx.lock().await;
    earnings_query::list(&mut **tx, page, per_page).await
  }

  async fn count_by_date(
    &self,
    from: chrono::DateTime<chrono::Utc>,
    to: chrono::DateTime<chrono::Utc>,
  ) -> Result<Vec<(chrono::NaiveDate, i64)>, RepositoryError> {
    let mut tx = self.tx.lock().await;
    earnings_query::count_by_date(&mut **tx, from, to).await
  }

  async fn insert_many(
    &self,
    items: &[earnings::Earnings],
    fingerprints: &[String],
  ) -> Result<Vec<earnings::EarningsRecord>, RepositoryError> {
    let mut tx = self.tx.lock().await;
    earnings_query::insert_many(&mut *tx, items, fingerprints).await
  }
}

#[async_trait]
impl NotifyQueueRepository for PgTxRepositories {
  async fn insert_monitor_marker(&self) -> Result<(), RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_queue::insert_monitor_marker(&mut **tx).await
  }

  async fn delete_monitor_marker(&self) -> Result<(), RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_queue::delete_monitor_marker(&mut **tx).await
  }

  async fn monitor_marker_exists(&self) -> Result<bool, RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_queue::monitor_marker_exists(&mut **tx).await
  }

  async fn replace_data_rows(
    &self,
    entries: &[subscription::NotifyQueueEntry],
  ) -> Result<(), RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_queue::replace_data_rows(&mut *tx, entries).await
  }

  async fn list_ready(&self) -> Result<Vec<subscription::NotifyQueueEntry>, RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_queue::list_ready(&mut **tx).await
  }

  async fn update_status(
    &self,
    id: i64,
    status: subscription::NotifyStatus,
  ) -> Result<(), RepositoryError> {
    let mut tx = self.tx.lock().await;
    notify_queue::update_status(&mut **tx, id, status).await
  }
}

impl RepositoryScope for PgTxRepositories {
  fn notify_group_repository(&mut self) -> &mut dyn NotifyGroupRepository {
    self
  }
  fn notify_discord_config_repository(&mut self) -> &mut dyn NotifyDiscordConfigRepository {
    self
  }
  fn notify_filter_repository(&mut self) -> &mut dyn NotifyFilterRepository {
    self
  }
  fn earnings_repository(&mut self) -> &mut dyn EarningsRepository {
    self
  }
  fn notify_queue_repository(&mut self) -> &mut dyn NotifyQueueRepository {
    self
  }
}

pub struct PgUnitOfWork {
  pool: PgPool,
}

impl PgUnitOfWork {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl UnitOfWork for PgUnitOfWork {
  async fn execute<F>(&self, f: F) -> Result<(), RepositoryError>
  where
    F: for<'a> FnOnce(&'a mut dyn RepositoryScope) -> BoxFuture<'a, Result<(), RepositoryError>>
      + Send
      + 'static,
  {
    let tx = self
      .pool
      .begin()
      .await
      .map_err(|e| RepositoryError::Other(e.to_string()))?;
    let mut scope = PgTxRepositories::new(tx);

    let result = f(&mut scope).await;

    let tx = scope.into_inner().await;
    match result {
      Ok(()) => {
        tx.commit()
          .await
          .map_err(|e| RepositoryError::Other(e.to_string()))?;
        Ok(())
      }
      Err(e) => {
        let _ = tx.rollback().await;
        Err(e)
      }
    }
  }
}
