/*
backend/crates/app/src/import/validate_row.rs
インポートの異常検知を定義
*/

// 内部ライブラリ
use config::ImportSettings;

pub struct NormalizedRow {
  pub ticker: String,
  pub company_name: String,
  pub notes: Option<String>,
  pub enabled: bool,
  pub enabled_was_missing: bool,
}

pub enum RowOutcome {
  /// 行全体が空(必須列も含め何も入力されていない)
  EmptySkip,
  /// 必須列が不完全(Ticker/CompanyNameの片方だけ、または全体一括設定でGroupNameが空)
  Broken { reason: String },
  /// 有効な行(warningsは異常値検知の結果、0件のこともある)
  Valid {
    row: NormalizedRow,
    warnings: Vec<String>,
  },
}

/// 1行を分類する。group_nameは全体一括設定の場合のみSome(呼び出し側で判定して渡す)。
pub fn classify_row(
  ticker: &str,
  company_name: &str,
  notes: Option<&str>,
  enabled: Option<bool>,
  group_name: Option<&str>, // 全体一括設定の場合のみチェック対象
  settings: &ImportSettings,
) -> RowOutcome {
  let ticker_trimmed = ticker.trim();
  let company_name_trimmed = company_name.trim();
  let group_name_trimmed = group_name.map(str::trim);

  let ticker_empty = ticker_trimmed.is_empty();
  let company_name_empty = company_name_trimmed.is_empty();
  let group_name_empty = group_name_trimmed.map(str::is_empty).unwrap_or(false);

  // 行全体が空(import-export.md 7章)
  if ticker_empty && company_name_empty && group_name.map(|g| g.trim().is_empty()).unwrap_or(true) {
    return RowOutcome::EmptySkip;
  }

  // 必須列の片方だけ入力(import-export.md 7章)
  if ticker_empty != company_name_empty {
    return RowOutcome::Broken {
      reason: "TickerとCompanyNameのどちらかが欠落しています".to_string(),
    };
  }

  // 本書3.3節: 全体一括設定でGroupNameが空の場合もエラー行とする
  if group_name.is_some() && group_name_empty {
    return RowOutcome::Broken {
      reason: "GroupNameが空です".to_string(),
    };
  }

  let ticker = earnings::normalize_ticker(ticker_trimmed);
  let company_name = company_name_trimmed.to_string();
  let notes = notes
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .map(str::to_string);

  let mut warnings = Vec::new();

  // 異常値検知(エラーにはせず警告として継続許可)
  if ticker.chars().count() > settings.ticker_max_len {
    warnings.push(format!(
      "tickerが{}文字を超えています",
      settings.ticker_max_len
    ));
  }
  if company_name.chars().count() > settings.company_name_max_len {
    warnings.push(format!(
      "company_nameが{}文字を超えています",
      settings.company_name_max_len
    ));
  }
  if let Some(ref n) = notes {
    if n.chars().count() > settings.notes_max_len {
      warnings.push(format!(
        "notesが{}文字を超えています",
        settings.notes_max_len
      ));
    }
  }

  let enabled_was_missing = enabled.is_none();
  if enabled_was_missing {
    warnings.push("EarningsWatch_Enabledが未入力のためtrueとして扱いました".to_string());
  }

  RowOutcome::Valid {
    row: NormalizedRow {
      ticker,
      company_name,
      notes,
      enabled: enabled.unwrap_or(true), // 未入力はtrue扱い
      enabled_was_missing,
    },
    warnings,
  }
}
