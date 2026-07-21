/*
backend/crates/app/src/import/import_filters_for_group.rs
グループごとの一括インポート
*/

// 標準ライブラリ
use std::collections::HashSet;

// 外部クレート
use chrono::Utc;

// 内部ライブラリ
use config::ImportSettings;
use identity::{FilterId, GroupId, UserId};
use repository::{NotifyFilterRepository, NotifyGroupRepository};
use subscription::NotifyFilter;

// 自クレート
use super::validate_row::{RowOutcome, classify_row};
use super::{ImportErrorRowData, ImportFiltersResult, ImportWarningData};
use crate::AppError;

pub struct ImportGroupFilterRowInput {
  pub ticker: String,
  pub company_name: String,
  pub notes: Option<String>,
  pub enabled: Option<bool>,
}

pub async fn import_filters_for_group(
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  import_settings: &ImportSettings,
  requester_user_id: UserId,
  group_id: GroupId,
  rows: Vec<ImportGroupFilterRowInput>,
  dry_run: bool,
) -> Result<ImportFiltersResult, AppError> {
  let group = group_repo
    .find_by_id(group_id)
    .await?
    .ok_or(AppError::NotFound)?;
  if group.user_id != requester_user_id {
    return Err(AppError::Forbidden);
  }

  let mut skipped_empty_rows = 0u32;
  let mut duplicate_count = 0u32;
  let mut error_rows = Vec::new();
  let mut warnings = Vec::new();
  let mut filters = Vec::new();
  let mut seen: HashSet<String> = HashSet::new();

  for (idx, input) in rows.into_iter().enumerate() {
    let row_number = (idx + 1) as u32;

    match classify_row(
      &input.ticker,
      &input.company_name,
      input.notes.as_deref(),
      input.enabled,
      None, // グループ単位インポートはgroup_name列を持たない(import-export.md 6章)
      import_settings,
    ) {
      RowOutcome::EmptySkip => skipped_empty_rows += 1,
      RowOutcome::Broken { reason } => error_rows.push(ImportErrorRowData { row_number, reason }),
      RowOutcome::Valid {
        row,
        warnings: row_warnings,
      } => {
        if !seen.insert(row.ticker.clone()) {
          duplicate_count += 1;
          continue;
        }

        for w in row_warnings {
          warnings.push(ImportWarningData {
            row_number,
            message: w,
          });
        }

        filters.push(NotifyFilter {
          id: FilterId::new(),
          group_id,
          ticker: row.ticker,
          company_name: row.company_name,
          notes: row.notes,
          enabled: row.enabled,
          created_at: Utc::now(),
        });
      }
    }
  }

  // import-export.md 9章: 有効な行が0件ならImportEmptyエラーとし、既存フィルタは変更しない
  if filters.is_empty() {
    return Err(AppError::ImportEmpty);
  }

  let imported_count = filters.len() as u32;

  if dry_run {
    return Ok(ImportFiltersResult {
      imported_count,
      skipped_empty_rows,
      duplicate_count,
      error_rows,
      created_groups: vec![], // グループ単位インポートでは常に空(import-export.md 10章)
      paused_groups: vec![],
      warnings,
    });
  }

  filter_repo
    .replace_all_for_group(group_id, &filters)
    .await?;

  Ok(ImportFiltersResult {
    imported_count,
    skipped_empty_rows,
    duplicate_count,
    error_rows,
    created_groups: vec![],
    paused_groups: vec![],
    warnings,
  })
}
