/*
backend/crates/app/src/group/test_send.rs
仮送信をするユースケース
*/
use earnings::EarningsEvaluation;
use identity::{GroupId, UserId};
use notifier::discord::EmbedColor;
use repository::{NotifyDiscordConfigRepository, NotifyGroupRepository};
use serde_json::json;
use subscription::NotifyMedium;

use crate::AppError;

pub struct TestSendInput {
  pub ticker: Option<String>,
  pub company_name: Option<String>,
  pub title: Option<String>,
  pub evaluation: Option<EarningsEvaluation>,
  pub embed_color: Option<String>,
  pub webhook_url: Option<String>, // 入力時は保存済み設定を一時上書き(DB非保存)
  pub mention_targets: Option<Vec<String>>,
}

pub struct TestSendOutput {
  pub success: bool,
  pub failure_reason: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub async fn test_send(
  group_repo: &dyn NotifyGroupRepository,
  discord_repo: &dyn NotifyDiscordConfigRepository,
  requester_user_id: UserId,
  group_id: GroupId,
  input: TestSendInput,
  webhook_enc_key: &[u8],
) -> Result<TestSendOutput, AppError> {
  let group = group_repo
    .find_by_id(group_id)
    .await?
    .ok_or(AppError::NotFound)?;
  if group.user_id != requester_user_id {
    return Err(AppError::Forbidden);
  }

  if group.medium != NotifyMedium::Discord {
    // Slackは仮実装のため、test-sendもDiscordのみ対象とする(引き継ぎメモ参照)
    return Ok(TestSendOutput {
      success: false,
      failure_reason: Some("Slack向けのテスト送信はまだ対応していません".to_string()),
    });
  }

  let row = discord_repo
    .find_by_group_id(group_id)
    .await?
    .ok_or(AppError::NotFound)?;

  // 未入力時はグループ保存済みの値を使う(02-types/api.md 7章、TestSendRequestの方針)
  let webhook_url = match input.webhook_url.or_else(|| {
    row.webhook_url_ciphertext.as_ref().and_then(|ciphertext| {
      let encrypted =
        crypto::Encrypted::<crypto::WebhookUrlTag>::from_ciphertext(ciphertext.clone());
      let aad = group_id.as_uuid().as_bytes();
      encrypted
        .decrypt(webhook_enc_key, aad)
        .ok()
        .map(|p| p.as_str().to_string())
    })
  }) {
    Some(url) if !url.is_empty() => url,
    _ => {
      return Ok(TestSendOutput {
        success: false,
        failure_reason: Some("webhook_urlが設定されていません".to_string()),
      });
    }
  };

  let ticker = input.ticker.unwrap_or_else(|| "0000".to_string());
  let company_name = input
    .company_name
    .unwrap_or_else(|| "サンプル株式会社".to_string());
  let title = input
    .title
    .unwrap_or_else(|| "【テスト送信】決算速報プレビュー".to_string());
  let evaluation = input.evaluation.unwrap_or(EarningsEvaluation::Unrated);
  let embed_color = input
    .embed_color
    .and_then(|s| EmbedColor::from_hex_string(&s).ok())
    .unwrap_or(EmbedColor::DEFAULT);

  // 最小限のDiscord Embedペイロード(本書3.4節、Phase 11で本処理用に拡張する前提の土台)
  let payload = json!({
      "embeds": [{
          "title": title,
          "description": format!("{company_name}({ticker}) - {evaluation:?}"),
          "color": u32::from_str_radix(embed_color.to_hex_string().trim_start_matches("0x"), 16).unwrap_or(0x87CEEB),
      }]
  });

  let client = reqwest::Client::new();
  match client.post(&webhook_url).json(&payload).send().await {
    Ok(response) if response.status().is_success() => Ok(TestSendOutput {
      success: true,
      failure_reason: None,
    }),
    Ok(response) => Ok(TestSendOutput {
      success: false,
      failure_reason: Some(format!("送信先が{}を返しました", response.status())),
    }),
    Err(e) => Ok(TestSendOutput {
      success: false,
      failure_reason: Some(format!("通信に失敗しました: {e}")),
    }),
  }
}
