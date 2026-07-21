/*
backend/crates/repository/src/unit_of_work.rs
複数Repository呼び出しをまたぐ一連の操作を
単一のDBトランザクションとしてアトミックに実行するための抽象型を定義
*/

// 標準ライブラリ
use std::future::Future;
use std::pin::Pin;

// 外部クレート
use async_trait::async_trait;

/// 自クレート
use crate::{
  EarningsRepository, NotifyDiscordConfigRepository, NotifyFilterRepository, NotifyGroupRepository,
  NotifyQueueRepository, NotifySlackConfigRepository, RepositoryResult,
};

/// futuresクレートを新規に追加しないよう
/// repositoryクレート内でローカルに定義した
/// Box化されたFutureのエイリアス。
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// トランザクション内で利用できるRepository群への参照をまとめたもの。
/// infra側が「このトランザクションに紐づくRepository実装」を集めて渡す。
/// 必要になったRepositoryをここへ随時追加していく想定(Phase 4時点では
/// 複数集約にまたがる操作が明確な範囲、すなわちグループ作成・CSVインポート・
/// monitor実行に必要なものに絞って先行定義している)。
pub trait RepositoryScope: Send {
  fn notify_group_repository(&mut self) -> &mut dyn NotifyGroupRepository;
  fn notify_discord_config_repository(&mut self) -> &mut dyn NotifyDiscordConfigRepository;
  fn notify_slack_config_repository(&mut self) -> &mut dyn NotifySlackConfigRepository;
  fn notify_filter_repository(&mut self) -> &mut dyn NotifyFilterRepository;
  fn earnings_repository(&mut self) -> &mut dyn EarningsRepository;
  fn notify_queue_repository(&mut self) -> &mut dyn NotifyQueueRepository;
}

/// 複数Repository呼び出しをまたぐ一連の操作を、単一のDBトランザクションとして
/// アトミックに実行するための抽象(本書3.5節)。
///
/// 例(Phase 8): グループ作成時、notify_groups行の挿入とnotify_discord_configs行の
/// 挿入を1トランザクションでアトミックに行う。
/// 例(Phase 9): CSVインポートの差分適用(削除+一括追加)を1トランザクションで行う。
/// 例(Phase 11): monitor完了時、既存決算データ行の削除+新規追加+マーカー行削除を
/// 1トランザクションで行う。
#[async_trait]
pub trait UnitOfWork: Send + Sync {
  async fn execute(
    &self,
    f: Box<
      dyn for<'a> FnOnce(&'a mut dyn RepositoryScope) -> BoxFuture<'a, RepositoryResult<()>> + Send,
    >,
  ) -> RepositoryResult<()>;
}
