/*
backend/crates/scraper/src/debug/debug.rs
デバッグ用スクレイパーのスクレイパー定義
*/

// 標準ライブラリ
use std::collections::HashSet;

// 外部クレート
use async_trait::async_trait;

// 内部ライブラリ
use earnings::{Earnings, EarningsEvaluation, compute_fingerprint};

// 自クレート
use super::models::*;
use crate::ScraperService;
use crate::error::{ScraperError, ScraperResult};

// デバッグ用スクレイパー
pub struct DebugScraper;

#[async_trait]
impl ScraperService for DebugScraper {
  async fn fetch_earning_info(
    &self,
    known_fingerprints: HashSet<String>,
  ) -> ScraperResult<(Vec<Earnings>, Vec<String>)> {
    let mut new_items = Vec::new();
    let mut page = 1u32;

    // 一覧ページの取得処理
    // 新規ページがなくなるまでループ取得する
    // fingerprintは一覧段階でのみ計算する
    loop {
      let items = fetch_list(page).await?;

      if items.is_empty() {
        break;
      }

      // 全て新規だったかのフラグ
      let mut all_new_this_page = true;

      for item in items {
        // 決算評価を列挙型に変換
        let evaluation = EarningsEvaluation::parse_from_site_text(&item.fingerprint_item_3);

        let fingerprint = compute_fingerprint(&[
          &item.fingerprint_item_1,
          &item.fingerprint_item_2,
          &format!("{evaluation:?}"),
        ]);

        // 既知のfingerprintだったらスキップ
        if known_fingerprints.contains(&fingerprint) {
          all_new_this_page = false;
          continue;
        }

        new_items.push((item, fingerprint, evaluation));
      }

      // 既知が1件でもあればページ送り打ち切り
      if !all_new_this_page {
        break;
      }

      page += 1;

      // 本来はここでクールタイムを設ける(サーバ負荷対策)
    }

    // ---- 新規分のみ詳細ページへ遷移する ----
    let mut new_earnings: Vec<Earnings> = Vec::with_capacity(new_items.len());
    let mut new_fingerprints: Vec<String> = Vec::with_capacity(new_items.len());

    for (item, fingerprint, evaluation) in new_items {
      let mut detail = fetch_detail(&item.url).await?;

      // 一覧段階のevaluationを最終的な保存値としても採用する(fingerprintとの整合性維持)
      detail.evaluation = evaluation;

      new_earnings.push(detail);
      new_fingerprints.push(fingerprint);

      // 本来はここでクールタイムを設ける(サーバ負荷対策)
    }

    Ok((new_earnings, new_fingerprints))
  }
}

async fn fetch_list(page: u32) -> ScraperResult<Vec<DebugListItem>> {
  // 一覧ページを取得
  let output = tokio::process::Command::new("python3")
    .arg("scripts/debug/debug.py")
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

  // DebugListOutputにパース
  let parsed: DebugListOutput =
    serde_json::from_slice(&output.stdout).map_err(|e| ScraperError::ParseFailed(e.to_string()))?;

  // 一覧リストのアイテムを返す
  Ok(parsed.items)
}

async fn fetch_detail(url: &str) -> ScraperResult<Earnings> {
  // 個別ページのスクレイピング処理
  let output = tokio::process::Command::new("python3")
    .arg("scripts/debug/debug.py")
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

  let parsed: DebugDetailOutput =
    serde_json::from_slice(&output.stdout).map_err(|e| ScraperError::ParseFailed(e.to_string()))?;

  Ok(Earnings {
    ticker: earnings::normalize_ticker(&parsed.ticker),
    company_name: parsed.company_name,
    published_at: parsed.published_at,
    title: parsed.title,
    url: parsed.url,
    summary: parsed.summary,
    // ここでのevaluationは暫定値
    // run_monitor側で一覧段階の値に上書きする
    evaluation: EarningsEvaluation::parse_from_site_text(&parsed.evaluation),
  })
}
