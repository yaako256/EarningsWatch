/*
backend/crates/repository/src/lib.rs
repositoryクレート
Repository Trait / UnitOfWork Trait
SQL・sqlxには依存しない
*/

mod earnings_repository;
mod error;
mod notify_config_repository;
mod notify_filter_repository;
mod notify_group_repository;
mod notify_history_repository;
mod notify_queue_repository;
mod page_repository;
mod refresh_token_repository;
mod system_notify_config_repository;
mod system_run_repository;
mod unit_of_work;
mod user_repository;
mod user_settings_repository;

pub use earnings_repository::{EarningsListFilter, EarningsRepository};
pub use error::{RepositoryError, RepositoryResult};
pub use notify_config_repository::{
  discord::{NotifyDiscordConfigRepository, NotifyDiscordConfigRow},
  slack::{NotifySlackConfigRepository, NotifySlackConfigRow},
};
pub use notify_filter_repository::{FilterCountBreakdown, NotifyFilterRepository};
pub use notify_group_repository::NotifyGroupRepository;
pub use notify_history_repository::NotifyHistoryRepository;
pub use notify_queue_repository::NotifyQueueRepository;
pub use page_repository::PageRepository;
pub use refresh_token_repository::RefreshTokenRepository;
pub use system_notify_config_repository::SystemNotifyConfigRepository;
pub use system_run_repository::SystemRunRepository;
pub use unit_of_work::{BoxFuture, RepositoryScope, UnitOfWork};
pub use user_repository::UserRepository;
pub use user_settings_repository::UserSettingsRepository;
