/*
backend/crates/app/src/export/export_earnings.rs
決算情報をエクスポートするユースケース
*/
use chrono::{DateTime, Utc};
use earnings::EarningsEvaluation;
use repository::EarningsRepository;
use rust_xlsxwriter::Workbook;

use crate::AppError;

/// エクスポートに必要な最小限の絞り込みのみ実装する(本書3.5節、一覧APIそのものは対象外)。
pub struct ExportEarningsFilter {
  pub ticker: Option<String>,
  pub company_name: Option<String>,
  pub evaluation: Option<EarningsEvaluation>,
  pub from: Option<DateTime<Utc>>,
  pub to: Option<DateTime<Utc>>,
}

pub async fn export_earnings(
  earnings_repo: &dyn EarningsRepository,
  filter: ExportEarningsFilter,
) -> Result<Vec<u8>, AppError> {
  // Phase 5のEarningsRepository::listは(page, per_page)方式のため、
  // エクスポート用に全件走査する(件数が個人・家族・友人向け規模を想定するため許容する)。
  let mut all_records = Vec::new();
  let mut page = 1u32;
  const PER_PAGE: u32 = 500;

  loop {
    let (records, total_count) = earnings_repo.list(page, PER_PAGE).await?;
    let fetched_all = records.is_empty();
    all_records.extend(records);
    if fetched_all || all_records.len() as i64 >= total_count {
      break;
    }
    page += 1;
  }

  let filtered: Vec<_> = all_records
    .into_iter()
    .filter(|r| {
      filter
        .ticker
        .as_ref()
        .map(|t| &r.ticker == t)
        .unwrap_or(true)
    })
    .filter(|r| {
      filter
        .company_name
        .as_ref()
        .map(|c| r.company_name.contains(c.as_str()))
        .unwrap_or(true)
    })
    .filter(|r| filter.evaluation.map(|e| r.evaluation == e).unwrap_or(true))
    .filter(|r| {
      filter
        .from
        .map(|from| r.published_at >= from)
        .unwrap_or(true)
    })
    .filter(|r| filter.to.map(|to| r.published_at < to).unwrap_or(true))
    .collect();

  let mut workbook = Workbook::new();
  let sheet = workbook.add_worksheet();

  sheet
    .write_string(0, 0, "EarningsWatch_Ticker")
    .map_err(xlsx_err)?;
  sheet
    .write_string(0, 1, "EarningsWatch_CompanyName")
    .map_err(xlsx_err)?;
  sheet
    .write_string(0, 2, "EarningsWatch_PublishedAt")
    .map_err(xlsx_err)?;
  sheet
    .write_string(0, 3, "EarningsWatch_Title")
    .map_err(xlsx_err)?;
  sheet
    .write_string(0, 4, "EarningsWatch_Url")
    .map_err(xlsx_err)?;
  sheet
    .write_string(0, 5, "EarningsWatch_Summary")
    .map_err(xlsx_err)?;
  sheet
    .write_string(0, 6, "EarningsWatch_Evaluation")
    .map_err(xlsx_err)?;

  for (idx, r) in filtered.into_iter().enumerate() {
    let row_idx = (idx + 1) as u32;
    sheet
      .write_string(row_idx, 0, &r.ticker)
      .map_err(xlsx_err)?;
    sheet
      .write_string(row_idx, 1, &r.company_name)
      .map_err(xlsx_err)?;
    sheet
      .write_string(row_idx, 2, &r.published_at.to_rfc3339())
      .map_err(xlsx_err)?;
    sheet.write_string(row_idx, 3, &r.title).map_err(xlsx_err)?;
    sheet.write_string(row_idx, 4, &r.url).map_err(xlsx_err)?;
    sheet
      .write_string(row_idx, 5, &r.summary)
      .map_err(xlsx_err)?;
    sheet
      .write_string(row_idx, 6, &format!("{:?}", r.evaluation))
      .map_err(xlsx_err)?;
  }

  workbook.save_to_buffer().map_err(xlsx_err)
}

fn xlsx_err(e: impl std::fmt::Display) -> AppError {
  AppError::InvalidInput(format!("Excelファイルの生成に失敗しました: {e}"))
}
