/*
backend/crates/notifier/src/discord/send.rs
discordの送信を定義
*/

// 外部クレート
use serde_json::json;

// 内部ライブラリ
use super::embed_color::EmbedColor;
use super::mention::{MentionTarget, TimeStyle};

pub struct DiscordMessageInput<'a> {
  pub ticker: &'a str,
  pub company_name: &'a str,
  pub title: &'a str,
  pub summary: &'a str,
  pub url: &'a str,
  pub evaluation_label: &'a str,
  pub embed_color: EmbedColor,
  pub mention_targets: &'a [String],
}

pub async fn send_discord_message(
  webhook_url: &str,
  input: &DiscordMessageInput<'_>,
) -> Result<(), NotifierError> {
  let (content, allowed_mentions) = build_mention_content(input.mention_targets);

  let payload = json!({
      "content": content,
      "allowed_mentions": allowed_mentions,
      "embeds": [{
          "title": input.title,
          "url": input.url,
          "description": format!("{}({})\n{}", input.company_name, input.ticker, input.summary),
          "color": color_to_u32(input.embed_color),
          "footer": { "text": input.evaluation_label },
      }]
  });

  let client = reqwest::Client::new();
  let response = client
    .post(webhook_url)
    .json(&payload)
    .send()
    .await
    .map_err(|e| NotifierError::SendFailed(e.to_string()))?;

  if response.status().is_success() {
    Ok(())
  } else {
    Err(NotifierError::SendFailed(format!(
      "送信先がステータス{}を返しました",
      response.status()
    )))
  }
}

/// mention_targetsをDiscordのcontent文字列とallowed_mentionsへ変換する
/// (design/03-features/notification.md 8章の変換表通り)。
fn build_mention_content(mention_targets: &[String]) -> (String, serde_json::Value) {
  let mut content_parts = Vec::new();
  let mut users = Vec::new();
  let mut roles = Vec::new();
  let mut parse_everyone = false;

  for raw in mention_targets {
    match MentionTarget::parse(raw) {
      Ok(MentionTarget::User(id)) => {
        content_parts.push(format!("<@{id}>"));
        users.push(id);
      }
      Ok(MentionTarget::Role(id)) => {
        content_parts.push(format!("<@&{id}>"));
        roles.push(id);
      }
      Ok(MentionTarget::Everyone) => {
        content_parts.push("@everyone".to_string());
        parse_everyone = true;
      }
      Ok(MentionTarget::Here) => {
        content_parts.push("@here".to_string());
        parse_everyone = true; // @hereもeveryoneパースフラグで許可される(notification.md 8章)
      }
      Ok(MentionTarget::Time(style)) => {
        let unix = chrono::Utc::now().timestamp();
        content_parts.push(format!("<t:{unix}:{}>", time_style_char(style)));
      }
      Err(e) => {
        // 不正な要素は警告ログを残しつつスキップし、送信自体は続行する(notification.md 8章)
        tracing::warn!(target: "notifier", raw = %e.raw, "不正なmention_targets要素をスキップしました");
      }
    }
  }

  let parse = if parse_everyone {
    vec!["everyone"]
  } else {
    vec![]
  };
  let content = content_parts.join(" ");
  let allowed_mentions = json!({ "parse": parse, "users": users, "roles": roles });

  (content, allowed_mentions)
}

fn time_style_char(style: TimeStyle) -> &'static str {
  match style {
    TimeStyle::T => "t",
    TimeStyle::LongT => "T",
    TimeStyle::D => "d",
    TimeStyle::LongD => "D",
    TimeStyle::F => "f",
    TimeStyle::LongF => "F",
    TimeStyle::R => "R",
  }
}

fn color_to_u32(color: EmbedColor) -> u32 {
  ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
}

#[derive(Debug, thiserror::Error)]
pub enum NotifierError {
  #[error("送信に失敗しました: {0}")]
  SendFailed(String),
}
