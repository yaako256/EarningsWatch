/*
backend/crates/earnings/src/fingerprint.rs
決算情報のfingperprintを生成するメソッドの定義
*/

// 外部クレート
// Hash
use sha2::{Digest, Sha256};

// 自クレート
use crate::EarningsEvaluation;

/// fingerprintを生成する。
/// 一覧ページ取得時点の判別用フィールド群(タイトル・書き出し・決算評価等)から
/// ハッシュ化する。
///
/// scraperクレートのサイト固有型(EarningItems等)をearningsクレートが
/// 知る必要がないよう、プリミティブな引数(&str配列)を取る形にしている。
/// サイトごとに判別用フィールド数が異なっても対応できるよう、
/// 固定引数ではなくスライスで受け取る。
pub fn compute_fingerprint(fingerprint_items: &[&str]) -> String {
  let mut hasher = Sha256::new();

  // 表記揺れ(前後の空白等)による誤った新規判定を避けるため、trimしてから結合する
  for item in fingerprint_items {
    hasher.update(item.trim().as_bytes());
    hasher.update([0u8]); // 項目間の区切り(結合時の意図しない衝突を避けるためのセパレータ)
  }

  // Hash化して返す
  format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn same_input_produces_same_fingerprint() {
    let a = compute_fingerprint(&[
      "タイトル",
      "本文",
      &format!("{:?}", EarningsEvaluation::Positive),
    ]);
    let b = compute_fingerprint(&[
      "タイトル",
      "本文",
      &format!("{:?}", EarningsEvaluation::Positive),
    ]);
    assert_eq!(a, b);
  }

  #[test]
  fn different_evaluation_produces_different_fingerprint() {
    let a = compute_fingerprint(&[
      "タイトル",
      "本文",
      &format!("{:?}", EarningsEvaluation::Unrated),
    ]);
    let b = compute_fingerprint(&[
      "タイトル",
      "本文",
      &format!("{:?}", EarningsEvaluation::Positive),
    ]);
    assert_ne!(a, b);
  }
}
