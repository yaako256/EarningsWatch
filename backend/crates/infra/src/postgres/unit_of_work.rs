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
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::Mutex;

// 内部ライブラリ
use repository::{
  BoxFuture, EarningsRepository, NotifyDiscordConfigRepository, NotifyDiscordConfigRow,
  NotifyFilterRepository, NotifyGroupRepository, NotifyQueueRepository, RepositoryError,
  RepositoryScope, UnitOfWork,
};

/// 1トランザクション内で動く全Repositoryを1つの構造体にまとめ、
/// tokio::sync::Mutexでトランザクションへの排他アクセスを行う(本書3.2節)。
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
    // 実装はcrates/infra/src/postgres/notify_group_repository.rsのSQLと同一のクエリを、
    // executorとして`&mut **tx`(PgPoolの代わりにトランザクション)を渡す形で呼び出す。
    // (重複するため詳細は省略。実装時は6.2節のSQLをそのまま流用する)
    todo!("6.2節のfind_by_id同様のクエリを &mut **tx に対して実行する")
  }

  // list_by_user_id / list_all / insert / update / deleteも同様にexecutorを`&mut **tx`に差し替えて実装する(以下省略)
}

// NotifyDiscordConfigRepository / NotifyFilterRepository / EarningsRepository / NotifyQueueRepository も
// 同様に「6章の実装のSQLをそのまま、executorだけ&mut **txに差し替えて」PgTxRepositoriesに実装していく。

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
