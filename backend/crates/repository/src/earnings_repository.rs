/*
backend/crates/repository/src/earnings_repository.rs
決算情報テーブルを司るリポジトリ
*/

// 外部クレート
use async_trait::async_trait;
use chrono::NaiveDate;

// 内部ライブラリ
use earnings::{Earnings, EarningsRecord};

// 自クレート
use crate::RepositoryResult;

pub struct EarningsListFilter {
  pub ticker: Option<String>,
  pub company_name: Option<String>,
  pub evaluation: Option<earnings::EarningsEvaluation>,
  pub from: Option<chrono::DateTime<chrono::Utc>>,
  pub to: Option<chrono::DateTime<chrono::Utc>>,
}

/// 決算情報テーブルのリポジトリ型
#[async_trait]
pub trait EarningsRepository: Send + Sync {
  async fn find_by_fingerprint(
    &self,
    fingerprint: &str,
  ) -> RepositoryResult<Option<EarningsRecord>>;
  /// 直近N件のfingerprintのみを取得する(design/03-features/scraping.md 8章、件数ベース1本化)
  async fn list_recent_fingerprints(&self, limit: u32) -> RepositoryResult<Vec<String>>;
  async fn list(&self, page: u32, per_page: u32) -> RepositoryResult<(Vec<EarningsRecord>, i64)>;
  /// 決算発表日ごとの件数集計(02-types/api.md 5章 summary、date_jstはapp層でJST変換して集計する)
  async fn count_by_date(
    &self,
    from: chrono::DateTime<chrono::Utc>,
    to: chrono::DateTime<chrono::Utc>,
  ) -> RepositoryResult<Vec<(chrono::NaiveDate, i64)>>;
  /// monitor実行時、新規決算をfingerprintと共に一括保存する(EarningsからEarningsRecordが生成される)
  async fn insert_many(
    &self,
    items: &[Earnings],
    fingerprints: &[String],
  ) -> RepositoryResult<Vec<EarningsRecord>>;

  /// DB側で絞り込み条件を適用する一覧取得(GET /api/earningsとエクスポート双方で使う)
  async fn list_filtered(
    &self,
    filter: &EarningsListFilter,
    page: u32,
    per_page: u32,
  ) -> RepositoryResult<(Vec<earnings::EarningsRecord>, i64)>;

  /// 本書4.5節、AdminDashboardResponse.total_earnings_count用
  async fn count_all(&self) -> RepositoryResult<i64>;

  /// JST基準の日別集計(DBクエリ側でAT TIME ZONE変換)
  async fn summary_daily_counts_jst(
    &self,
    from: Option<chrono::DateTime<chrono::Utc>>,
    to: Option<chrono::DateTime<chrono::Utc>>,
  ) -> RepositoryResult<Vec<(NaiveDate, i64)>>;
}
