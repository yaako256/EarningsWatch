/*
backend/crates/notifier/src/discord/mention.rs
Discordのメンションの型定義
*/
use serde::{Deserialize, Serialize};

/// 時間のメンション種類の列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeStyle {
  T,
  LongT,
  D,
  LongD,
  F,
  LongF,
  R,
}

/// メンション種類の列挙型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MentionTarget {
  User(String),
  Role(String),
  Everyone,
  Here,
  Time(TimeStyle),
}

impl MentionTarget {
  // 不正な文字列は警告ログを残しつつスキップする方針(呼び出し側で対応、design/02-types/notifier.md 1章)
  pub fn parse(_raw: &str) -> Result<Self, ParseMentionTargetError> {
    todo!("プレフィックス(user:/role:/time:)またはeveryone/hereでの機械的な判別(実装はPhase 5以降)")
  }
}

/// メンション系統ののエラー型
#[derive(Debug, thiserror::Error)]
#[error("mention_targets要素の形式が不正です: {raw}")]
pub struct ParseMentionTargetError {
  pub raw: String,
}
