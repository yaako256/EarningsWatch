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

  pub fn from_hex_string(s: &str) -> Result<Self, ParseColorError> {
    let hex = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    if hex.len() != 6 {
      return Err(ParseColorError);
    }
    let value = u32::from_str_radix(hex, 16).map_err(|_| ParseColorError)?;
    Ok(Self {
      r: ((value >> 16) & 0xFF) as u8,
      g: ((value >> 8) & 0xFF) as u8,
      b: (value & 0xFF) as u8,
    })
  }

  pub fn to_hex_string(&self) -> String {
    format!("0x{:02X}{:02X}{:02X}", self.r, self.g, self.b)
  }
}

#[derive(Debug, thiserror::Error)]
#[error("embed_colorの形式が不正です")]
pub struct ParseColorError;
