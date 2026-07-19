/*
backend/crates/earnings/src/fingerprint.rs
決算情報のfingperprintを生成するメソッドの定義
*/

// 外部クレート
// Hash
use sha2::{Digest, Sha256};

// 自クレート
use crate::EarningsEvaluation;

/// fingerprintを生成する(design/03-features/scraping.md 3章)。
/// 一覧ページ取得時点の判別用フィールド3項目(タイトル・書き出し・決算評価)から
/// ハッシュ化する。証券コード・公開時刻は意図的に含めない。
///
/// scraperクレートのサイト固有型(EarningItems等)をearningsクレートが
/// 知る必要がないよう、プリミティブな引数を取る形にしている。
pub fn compute_fingerprint(
  title: &str,
  body_excerpt: &str,
  evaluation: EarningsEvaluation,
) -> String {
  let mut hasher = Sha256::new();
  // 表記揺れ(前後の空白等)による誤った新規判定を避けるため、trimしてから結合する
  hasher.update(title.trim().as_bytes());
  hasher.update([0u8]); // 項目間の区切り(結合時の意図しない衝突を避けるためのセパレータ)
  hasher.update(body_excerpt.trim().as_bytes());
  hasher.update([0u8]);
  hasher.update(format!("{evaluation:?}").as_bytes());

  // Hash化して返す
  format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn same_input_produces_same_fingerprint() {
    let a = compute_fingerprint("タイトル", "本文", EarningsEvaluation::Positive);
    let b = compute_fingerprint("タイトル", "本文", EarningsEvaluation::Positive);
    assert_eq!(a, b);
  }

  #[test]
  fn different_evaluation_produces_different_fingerprint() {
    let a = compute_fingerprint("タイトル", "本文", EarningsEvaluation::Unrated);
    let b = compute_fingerprint("タイトル", "本文", EarningsEvaluation::Positive);
    assert_ne!(a, b);
  }
}
