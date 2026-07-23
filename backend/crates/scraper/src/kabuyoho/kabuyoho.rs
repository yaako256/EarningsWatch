/*
backend/crates/scraper/src/kabuyoho/kabuyoho.rs
株予報Proのスクレイピンパー処理
*/

// 標準ライブラリ
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use earnings::{Earnings, EarningsEvaluation, compute_fingerprint};

// 自クレート
use super::models::*;
use crate::ScraperService;
use crate::error::{ScraperError, ScraperResult};

pub struct KabuyohoScraper {
  list_page_interval: Duration,
  detail_interval: Duration,
}

impl KabuyohoScraper {
  pub fn new(list_page_interval_seconds: u64, detail_interval_seconds: u64) -> Self {
    Self {
      list_page_interval: Duration::from_secs(list_page_interval_seconds),
      detail_interval: Duration::from_secs(detail_interval_seconds),
    }
  }
}

#[async_trait]
impl ScraperService for KabuyohoScraper {
  async fn fetch_earning_info(
    &self,
    known_fingerprints: HashSet<String>,
    scraper_dir_path: &Path,
  ) -> ScraperResult<(Vec<Earnings>, Vec<String>)> {
    let mut new_items = Vec::new();
    let mut page = 1u32;

    // ---- 一覧ページの取得処理(新規0件で終了) ----
    // fingerprintは一覧段階でのみ計算する
    loop {
      let items = fetch_list(page, scraper_dir_path).await?;

      if items.is_empty() {
        break;
      }

      // 全て新規だったかのフラグ
      let mut all_new_this_page = true;

      // fingerprint確認
      for item in items {
        // 決算評価を列挙型に変換
        let evaluation = EarningsEvaluation::parse_from_site_text(&item.fingerprint_evaluation);

        // fingerprint作成
        let fingerprint = compute_fingerprint(&[
          &item.fingerprint_title,
          &item.fingerprint_summary,
          &format!("{evaluation:?}"),
        ]);

        // 既知かどうかをHashSetで判別
        if known_fingerprints.contains(&fingerprint) {
          // 既知のfingerprintだったらスキップ
          all_new_this_page = false;
          continue;
        }

        new_items.push((item, fingerprint, evaluation));
      }

      // 各ページ取得の最後に必ず待機する
      tokio::time::sleep(self.list_page_interval).await;

      // 既知が1件でもあればページ送り打ち切り
      if !all_new_this_page {
        break;
      }

      page += 1;
    }

    // ---- 新規分のみ詳細ページへ遷移する ----
    let mut new_earnings: Vec<Earnings> = Vec::with_capacity(new_items.len());
    let mut new_fingerprints: Vec<String> = Vec::with_capacity(new_items.len());
    let total = new_items.len();

    for (idx, (item, fingerprint, evaluation)) in new_items.into_iter().enumerate() {
      // 個別ページから決算情報を取得
      let mut detail = fetch_detail(&item.url, scraper_dir_path).await?;

      // 一覧段階の値をfingerprintとの整合性のため採用
      detail.evaluation = evaluation;

      new_earnings.push(detail);
      new_fingerprints.push(fingerprint);

      // 最後のページでの処理は待機を行わない
      if idx + 1 < total {
        tokio::time::sleep(self.detail_interval).await;
      }
    }

    Ok((new_earnings, new_fingerprints))
  }
}

async fn fetch_list(page: u32, scraper_dir_path: &Path) -> ScraperResult<Vec<KabuyohoListItem>> {
  // Python スクリプトへのパスを組み立てる
  let script_path = scraper_dir_path.join("kabuyoho/kabuyoho.py");

  // 一覧ページを取得
  let output = tokio::process::Command::new("python3")
    .arg(&script_path)
    .arg("list")
    .arg("--page")
    .arg(page.to_string())
    .output()
    .await
    .map_err(|e| ScraperError::ProcessFailed(e.to_string()))?;

  // 成功したか
  if !output.status.success() {
    return Err(ScraperError::ProcessFailed(
      String::from_utf8_lossy(&output.stderr).to_string(),
    ));
  }

  // stderrに書かれた警告をtracingへ転送する(構造化はしない、事実のみ)
  forward_stderr_warnings(&output.stderr);

  // Jsonパース
  let parsed: KabuyohoListOutput =
    serde_json::from_slice(&output.stdout).map_err(|e| ScraperError::ParseFailed(e.to_string()))?;

  Ok(parsed.items)
}

async fn fetch_detail(url: &str, scraper_dir_path: &Path) -> ScraperResult<Earnings> {
  let script_path = scraper_dir_path.join("kabuyoho/kabuyoho.py");

  // 個別ページのスクレイピング処理
  let output = tokio::process::Command::new("python3")
    .arg(&script_path)
    .arg("detail")
    .arg("--url")
    .arg(url)
    .output()
    .await
    .map_err(|e| ScraperError::ProcessFailed(e.to_string()))?;

  // 成功したか
  if !output.status.success() {
    return Err(ScraperError::ProcessFailed(
      String::from_utf8_lossy(&output.stderr).to_string(),
    ));
  }

  // stderrに書かれた警告をtracingへ転送する
  forward_stderr_warnings(&output.stderr);

  // Jsonパース
  let parsed: KabuyohoDetailOutput =
    serde_json::from_slice(&output.stdout).map_err(|e| ScraperError::ParseFailed(e.to_string()))?;

  Ok(Earnings {
    ticker: earnings::normalize_ticker(&parsed.ticker),
    company_name: parsed.company_name,
    published_at: parsed.published_at,
    title: parsed.title,
    url: url.to_string(),
    summary: parsed.summary,
    evaluation: EarningsEvaluation::Unrated, // 呼び出し元(fetch_earning_info)で一覧段階の値に上書きされる
  })
}

/// Pythonのstderrをそのままtracing::warn!へ1行転送する(構造化なし)。
fn forward_stderr_warnings(stderr: &[u8]) {
  if stderr.is_empty() {
    return;
  }
  let text = String::from_utf8_lossy(stderr);
  tracing::warn!(target: "scraper::kabuyoho", "{}", text.trim());
}
