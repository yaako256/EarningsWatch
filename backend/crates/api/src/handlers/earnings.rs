/*
backend/crates/api/src/handlers/earnings.rs
決算情報系のハンドラ1
*/

// 外部クレート
use axum::Json;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use earnings::EarningsEvaluation;

// 自クレート
use crate::error::ApiAppError;
use crate::extractor::AuthUser;
use crate::handlers::common::ExportFormat;
use crate::response::{ApiResponse, Page as ApiPage};
use crate::state::AppState;

// ─── GET /api/earnings ───
#[derive(Deserialize)]
pub struct ListEarningsQuery {
  pub ticker: Option<String>,
  pub company_name: Option<String>,
  pub evaluation: Option<EarningsEvaluation>,
  pub from: Option<DateTime<Utc>>,
  pub to: Option<DateTime<Utc>>,
  pub page: u32,
  pub per_page: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsResponse {
  pub id: i64,
  pub ticker: String,
  pub company_name: String,
  pub published_at: DateTime<Utc>,
  pub title: String,
  pub url: String,
  pub summary: String,
  pub evaluation: EarningsEvaluation,
}

impl From<earnings::EarningsRecord> for EarningsResponse {
  fn from(r: earnings::EarningsRecord) -> Self {
    Self {
      id: r.id,
      ticker: r.ticker,
      company_name: r.company_name,
      published_at: r.published_at,
      title: r.title,
      url: r.url,
      summary: r.summary,
      evaluation: r.evaluation,
    }
  }
}

pub async fn list_earnings(
  State(state): State<AppState>,
  _auth_user: AuthUser,
  Query(query): Query<ListEarningsQuery>,
) -> Result<Json<ApiResponse<ApiPage<EarningsResponse>>>, ApiAppError> {
  let (records, total_count) = app::list_earnings(
    state.earnings_repository.as_ref(),
    repository::EarningsListFilter {
      ticker: query.ticker,
      company_name: query.company_name,
      evaluation: query.evaluation,
      from: query.from,
      to: query.to,
    },
    query.page,
    query.per_page,
  )
  .await?;

  let total_pages = ((total_count as f64) / (query.per_page as f64)).ceil() as u32;

  Ok(Json(ApiResponse::ok(ApiPage {
    items: records.into_iter().map(EarningsResponse::from).collect(),
    page: query.page,
    per_page: query.per_page,
    total_count,
    total_pages,
  })))
}

// ─── GET /api/earnings/summary ───
#[derive(Deserialize)]
pub struct EarningsSummaryQuery {
  pub from: Option<DateTime<Utc>>,
  pub to: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyEarningsCount {
  pub date_jst: NaiveDate,
  pub count: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsSummaryResponse {
  pub daily_counts: Vec<DailyEarningsCount>,
}

pub async fn earnings_summary(
  State(state): State<AppState>,
  _auth_user: AuthUser,
  Query(query): Query<EarningsSummaryQuery>,
) -> Result<Json<ApiResponse<EarningsSummaryResponse>>, ApiAppError> {
  let counts =
    app::earnings_summary(state.earnings_repository.as_ref(), query.from, query.to).await?;

  Ok(Json(ApiResponse::ok(EarningsSummaryResponse {
    daily_counts: counts
      .into_iter()
      .map(|(date_jst, count)| DailyEarningsCount {
        date_jst,
        count: count as u32,
      })
      .collect(),
  })))
}

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
