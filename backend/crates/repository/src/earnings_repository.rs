/*
backend/crates/repository/src/earnings_repository.rs
決算情報テーブルを司るリポジトリ
*/

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use earnings::{Earnings, EarningsRecord};

// 自クレート
use crate::RepositoryResult;

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
}
