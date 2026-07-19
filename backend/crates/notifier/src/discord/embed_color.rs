/*
backend/crates/notifier/src/discord/embed_color.rs
DiscordのEmbed色の型定義
*/

// 外部クレート
use serde::{Deserialize, Serialize};

/// Embed色の構造体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbedColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
}

impl EmbedColor {
  // config管理のデフォルト色(水色 0x87CEEB)。
  // configクレートのdefault_embed_colorと値を揃える。
  pub const DEFAULT: Self = Self {
    r: 0x87,
    g: 0xCE,
    b: 0xEB,
  };

  pub fn from_hex_string(_s: &str) -> Result<Self, ParseColorError> {
    todo!("\"0x87EB87\"形式の文字列からのパース(実装はPhase 5以降)")
  }

  pub fn to_hex_string(&self) -> String {
    format!("0x{:02X}{:02X}{:02X}", self.r, self.g, self.b)
  }
}

#[derive(Debug, thiserror::Error)]
#[error("embed_colorの形式が不正です")]
pub struct ParseColorError;
