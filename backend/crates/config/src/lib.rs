/*
backend/crates/config/src/lib.rs
設定の型定義とその読み込みを司る
*/
mod error;
mod loader;
mod setting;

pub use error::ConfigLoadError;
pub use loader::load;
pub use setting::*;
