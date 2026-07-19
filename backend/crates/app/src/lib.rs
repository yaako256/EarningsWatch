/*
backend/crates/app/src/lib.rs
appクレート
ユースケース層
*/

mod admin;
mod error;

pub use admin::create_admin_user;
pub use error::AppError;
