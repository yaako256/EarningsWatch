/*
backend/crates/api/src/handlers/earnings.rs
決算情報系のハンドラ1
*/

// 外部クレート
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use serde::Deserialize;

// 内部ライブラリ
use earnings::EarningsEvaluation;

// 自クレート
use crate::error::ApiAppError;
use crate::extractor::AuthUser;
use crate::handlers::common::ExportFormat;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ExportEarningsQuery {
  pub ticker: Option<String>,
  pub company_name: Option<String>,
  pub evaluation: Option<EarningsEvaluation>,
  pub from: Option<DateTime<Utc>>,
  pub to: Option<DateTime<Utc>>,
  pub format: ExportFormat,
}

pub async fn export_earnings(
  State(state): State<AppState>,
  _auth_user: AuthUser,
  Query(query): Query<ExportEarningsQuery>,
) -> Result<impl IntoResponse, ApiAppError> {
  let ExportFormat::Xlsx = query.format;

  let bytes = app::export_earnings(
    state.earnings_repository.as_ref(),
    app::ExportEarningsFilter {
      ticker: query.ticker,
      company_name: query.company_name,
      evaluation: query.evaluation,
      from: query.from,
      to: query.to,
    },
  )
  .await?;

  Ok((
    [
      (
        axum::http::header::CONTENT_TYPE,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
      ),
      (
        axum::http::header::CONTENT_DISPOSITION,
        "attachment; filename=\"earnings.xlsx\"".to_string(),
      ),
    ],
    axum::body::Bytes::from(bytes),
  ))
}
