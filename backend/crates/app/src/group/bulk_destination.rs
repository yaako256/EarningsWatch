/*
backend/crates/app/src/group/bulk_destination.rs
送信媒体ごとの設定の一括設定をするユースケース
*/

// 内部ライブラリ
use identity::{GroupId, UserId};
use repository::{
  NotifyDiscordConfigRepository, NotifyDiscordConfigRow, NotifyGroupRepository,
  NotifySlackConfigRepository, NotifySlackConfigRow,
};

// 自クレート
use crate::AppError;
use crate::group::group_config::GroupConfigData;

/// 送信媒体ごとの設定の一括設定をする
pub async fn bulk_destination(
  group_repo: &dyn NotifyGroupRepository,
  discord_repo: &dyn NotifyDiscordConfigRepository,
  slack_repo: &dyn NotifySlackConfigRepository,
  requester_user_id: UserId,
  group_ids: Vec<GroupId>,
  config: GroupConfigData,
  webhook_enc_key: &[u8],
) -> Result<u32, AppError> {
  // 更新数記録用
  let mut updated_count = 0u32;

  // 全グループでループ
  for group_id in group_ids {
    // 正しいグループか
    let group = match group_repo.find_by_id(group_id).await? {
      Some(g) if g.user_id == requester_user_id => g,
      _ => continue, // 所有者でない、または存在しないグループはスキップ(エラーにはしない)
    };

    // 一括設定(Unionで設定送信媒体を判別)
    match &config {
      // Discordの設定を変更
      GroupConfigData::Discord {
        webhook_url,
        embed_color,
        mention_enabled,
        mention_targets,
      } => {
        // 設定送信先がDiscordじゃなかったらスキップ
        if group.medium != subscription::NotifyMedium::Discord {
          continue;
        }

        let ciphertext = match &webhook_url {
          Some(url) if !url.is_empty() => {
            let aad = group_id.as_uuid().as_bytes();
            Some(
              crypto::Encrypted::<crypto::WebhookUrlTag>::encrypt(url, webhook_enc_key, aad)
                .map_err(|_| AppError::CryptoError)?
                .as_str()
                .to_string(),
            )
          }
          _ => None,
        };

        discord_repo
          .upsert(
            group_id,
            &NotifyDiscordConfigRow {
              webhook_url_ciphertext: ciphertext,
              embed_color: embed_color.clone(),
              mention_enabled: *mention_enabled,
              mention_targets: mention_targets.clone(),
            },
          )
          .await?;

        updated_count += 1;
      }
      // Slackの設定を変更
      GroupConfigData::Slack {
        webhook_url,
        mention_enabled,
        mention_targets,
      } => {
        // 設定送信先がSlackじゃなかったらスキップ
        if group.medium != subscription::NotifyMedium::Slack {
          continue;
        }
        let ciphertext = match webhook_url.as_ref() {
          Some(url) if !url.is_empty() => {
            let aad = group_id.as_uuid().as_bytes();
            Some(
              crypto::Encrypted::<crypto::WebhookUrlTag>::encrypt(url, webhook_enc_key, aad)
                .map_err(|_| AppError::CryptoError)?
                .as_str()
                .to_string(),
            )
          }
          _ => None,
        };
        slack_repo
          .upsert(
            group_id,
            &NotifySlackConfigRow {
              webhook_url_ciphertext: ciphertext,
              mention_enabled: *mention_enabled,
              mention_targets: mention_targets.clone(),
            },
          )
          .await?;
        updated_count += 1;
      }
    }
  }

  Ok(updated_count)
}
