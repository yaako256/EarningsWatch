/*
backend/crates/app/src/lib.rs
appクレート
ユースケース層
*/

mod admin;
mod auth;
mod error;
mod group;

pub use admin::create_admin_user;
pub use auth::{LoginOutput, login, logout, refresh};
pub use error::AppError;
pub use group::*;
