/*
backend/crates/app/src/dashboard/mod.rs
ダッシュボード関連のユースケース
*/
mod get_dashboard;
pub use get_dashboard::{DashboardData, get_dashboard};
