/*
backend/crates/app/src/admin/mod.rs
管理者系のユースケース
*/
// crates/app/src/admin/mod.rs
mod admin_dashboard;
mod create_admin_user;
mod create_temp_user;
mod disable_user;
mod list_logs;
mod list_users;
mod notify_config;
mod user_summary;

pub use admin_dashboard::{AdminDashboardData, admin_dashboard};
pub use create_admin_user::create_admin_user;
pub use create_temp_user::{CreateTempUserOutput, create_temp_user};
pub use disable_user::disable_user;
pub use list_logs::list_logs;
pub use list_users::list_users;
pub use notify_config::{NotifyConfigData, get_notify_config, update_notify_config};
pub use user_summary::{UserSummaryData, user_summary};
