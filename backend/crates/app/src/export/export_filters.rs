/*
backend/crates/app/src/export/export_filters.rs
フィルターの全体/グループごとのエクスポートのユースケース
*/
use identity::{GroupId, UserId};
use repository::{NotifyFilterRepository, NotifyGroupRepository};
use rust_xlsxwriter::Workbook;

use crate::AppError;

/// 全体一括エクスポート(GroupName列あり)。
pub async fn export_filters_all(
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  user_id: UserId,
) -> Result<Vec<u8>, AppError> {
  let groups = group_repo.list_by_user_id(user_id).await?;

  let mut workbook = Workbook::new();
  let sheet = workbook.add_worksheet();

  sheet
    .write_string(0, 0, "EarningsWatch_Ticker")
    .map_err(xlsx_err)?;
  sheet
    .write_string(0, 1, "EarningsWatch_CompanyName")
    .map_err(xlsx_err)?;
  sheet
    .write_string(0, 2, "EarningsWatch_GroupName")
    .map_err(xlsx_err)?;
  sheet
    .write_string(0, 3, "EarningsWatch_Notes")
    .map_err(xlsx_err)?;
  sheet
    .write_string(0, 4, "EarningsWatch_Enabled")
    .map_err(xlsx_err)?;

  let mut row_idx = 1u32;
  for group in &groups {
    let filters = filter_repo.list_by_group_id(group.id).await?;
    for f in filters {
      sheet
        .write_string(row_idx, 0, &f.ticker)
        .map_err(xlsx_err)?;
      sheet
        .write_string(row_idx, 1, &f.company_name)
        .map_err(xlsx_err)?;
      sheet
        .write_string(row_idx, 2, &group.name)
        .map_err(xlsx_err)?;
      sheet
        .write_string(row_idx, 3, f.notes.as_deref().unwrap_or(""))
        .map_err(xlsx_err)?;
      sheet
        .write_boolean(row_idx, 4, f.enabled)
        .map_err(xlsx_err)?;
      row_idx += 1;
    }
  }

  workbook.save_to_buffer().map_err(xlsx_err)
}

/// グループごとのエクスポート(GroupName列なし)。
pub async fn export_filters_for_group(
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  requester_user_id: UserId,
  group_id: GroupId,
) -> Result<Vec<u8>, AppError> {
  let group = group_repo
    .find_by_id(group_id)
    .await?
    .ok_or(AppError::NotFound)?;
  if group.user_id != requester_user_id {
    return Err(AppError::Forbidden);
  }

  let filters = filter_repo.list_by_group_id(group_id).await?;

  let mut workbook = Workbook::new();
  let sheet = workbook.add_worksheet();

  sheet
    .write_string(0, 0, "EarningsWatch_Ticker")
    .map_err(xlsx_err)?;
  sheet
    .write_string(0, 1, "EarningsWatch_CompanyName")
    .map_err(xlsx_err)?;
  sheet
    .write_string(0, 2, "EarningsWatch_Notes")
    .map_err(xlsx_err)?;
  sheet
    .write_string(0, 3, "EarningsWatch_Enabled")
    .map_err(xlsx_err)?;

  for (idx, f) in filters.into_iter().enumerate() {
    let row_idx = (idx + 1) as u32;
    sheet
      .write_string(row_idx, 0, &f.ticker)
      .map_err(xlsx_err)?;
    sheet
      .write_string(row_idx, 1, &f.company_name)
      .map_err(xlsx_err)?;
    sheet
      .write_string(row_idx, 2, f.notes.as_deref().unwrap_or(""))
      .map_err(xlsx_err)?;
    sheet
      .write_boolean(row_idx, 3, f.enabled)
      .map_err(xlsx_err)?;
  }

  workbook.save_to_buffer().map_err(xlsx_err)
}

fn xlsx_err(e: impl std::fmt::Display) -> AppError {
  AppError::InvalidInput(format!("Excelファイルの生成に失敗しました: {e}"))
}
