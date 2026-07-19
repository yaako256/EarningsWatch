/*
backend/crates/earnings/src/ticker.rs
証券コードの正規化を定義
".T"を含めない4文字に正規化する。
*/

/// 証券コードの接尾辞(例: ".T")を除去し正規化する。
/// DB保存など、表示のすべてでこの正規化後の値を使う。
/// CSVインポート等ユーザ入力にも同様に適用する。
pub fn normalize_ticker(raw: &str) -> String {
  // 正規化して返す
  raw.trim().split('.').next().unwrap_or(raw).to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn strips_suffix() {
    assert_eq!(normalize_ticker("7203.T"), "7203");
  }

  #[test]
  fn no_suffix_is_unchanged() {
    assert_eq!(normalize_ticker("7203"), "7203");
  }
}
