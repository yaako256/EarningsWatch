/*
backend/crates/scraper/src/traits.rs
スクレイピングのトレイト型を定義
*/

// 標準ライブラリ
use std::collections::HashSet;

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use earnings::Earnings;

// 自クレート
use crate::error::ScraperResult;

// #[async_trait]
// pub trait ScraperService: Send + Sync {
//   async fn fetch_list(&self, page: u32) -> Result<Vec<RawEarningItem>, ScraperError>;
//   async fn fetch_detail(&self, url: &str) -> Result<Earnings, ScraperError>;
// }

#[async_trait]
pub trait ScraperService: Send + Sync {
  async fn fetch_earning_info(
    &self,
    known_fingerprints: HashSet<String>,
  ) -> ScraperResult<(Vec<Earnings>, Vec<String>)>;
}
