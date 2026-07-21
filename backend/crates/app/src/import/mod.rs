/*
backend/crates/app/src/import/mod.rs
モジュールの公開定義とモジュール内共通構造体の定義
*/
mod import_filters_all;
mod import_filters_for_group;
mod validate_row;

pub use import_filters_all::{ImportFilterRowInput, import_filters_all};
pub use import_filters_for_group::{ImportGroupFilterRowInput, import_filters_for_group};

use identity::GroupId;

pub struct ImportErrorRowData {
  pub row_number: u32,
  pub reason: String,
}

pub struct ImportWarningData {
  pub row_number: u32,
  pub message: String,
}

pub struct GroupRefData {
  pub id: GroupId,
  pub name: String,
}

pub struct ImportFiltersResult {
  pub imported_count: u32,
  pub skipped_empty_rows: u32,
  pub duplicate_count: u32,
  pub error_rows: Vec<ImportErrorRowData>,
  pub created_groups: Vec<GroupRefData>,
  pub paused_groups: Vec<GroupRefData>,
  pub warnings: Vec<ImportWarningData>,
}
