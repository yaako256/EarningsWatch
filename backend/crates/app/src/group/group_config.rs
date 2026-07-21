/*
backend/crates/app/src/group/group_config.rs
グループごとの設定を司るユースケース群
*/

// 内部ライブラリ
use crypto::{Encrypted, WebhookUrlTag};
use identity::{GroupId, UserId};
use notifier::discord::EmbedColor;
use repository::{
  NotifyDiscordConfigRepository, NotifyDiscordConfigRow, NotifyGroupRepository,
  NotifySlackConfigRepository, NotifySlackConfigRow,
};
use subscription::NotifyMedium;

// 自クレート
use crate::AppError;

pub enum GroupConfigData {
  Discord {
    webhook_url: Option<String>, // 復号済み平文
    embed_color: Option<String>,
    mention_enabled: bool,
    mention_targets: Vec<String>,
  },
  Slack {
    webhook_url: Option<String>, // 復号済み平文
    mention_enabled: bool,
    mention_targets: Vec<String>,
  },
}

#[allow(clippy::too_many_arguments)]
pub async fn get_group_config(
  group_repo: &dyn NotifyGroupRepository,
  discord_repo: &dyn NotifyDiscordConfigRepository,
  slack_repo: &dyn NotifySlackConfigRepository,
  requester_user_id: UserId,
  group_id: GroupId,
  webhook_enc_key: &[u8],
) -> Result<GroupConfigData, AppError> {
  let group = group_repo
    .find_by_id(group_id)
    .await?
    .ok_or(AppError::NotFound)?;
  if group.user_id != requester_user_id {
    return Err(AppError::Forbidden);
  }

  match group.medium {
    NotifyMedium::Discord => {
      let row = discord_repo
        .find_by_group_id(group_id)
        .await?
        .ok_or(AppError::NotFound)?;
      let webhook_url = decrypt_webhook_url(row.webhook_url_ciphertext, group_id, webhook_enc_key)?;

      Ok(GroupConfigData::Discord {
        webhook_url,
        embed_color: row.embed_color,
        mention_enabled: row.mention_enabled,
        mention_targets: row.mention_targets,
      })
    }
    NotifyMedium::Slack => {
      let row = slack_repo
        .find_by_group_id(group_id)
        .await?
        .ok_or(AppError::NotFound)?;
      let webhook_url = decrypt_webhook_url(row.webhook_url_ciphertext, group_id, webhook_enc_key)?;

      Ok(GroupConfigData::Slack {
        webhook_url,
        mention_enabled: row.mention_enabled,
        mention_targets: row.mention_targets,
      })
    }
  }
}

#[allow(clippy::too_many_arguments)]
pub async fn put_group_config(
  group_repo: &dyn NotifyGroupRepository,
  discord_repo: &dyn NotifyDiscordConfigRepository,
  slack_repo: &dyn NotifySlackConfigRepository,
  requester_user_id: UserId,
  group_id: GroupId,
  config: GroupConfigData,
  webhook_enc_key: &[u8],
) -> Result<(), AppError> {
  let group = group_repo
    .find_by_id(group_id)
    .await?
    .ok_or(AppError::NotFound)?;
  if group.user_id != requester_user_id {
    return Err(AppError::Forbidden);
  }

  match config {
    GroupConfigData::Discord {
      webhook_url,
      embed_color,
      mention_enabled,
      mention_targets,
    } => {
      // embed_colorはNULLならデフォルト色として判定するため、
      // 空文字はNoneとして扱う(EmbedColor::from_hex_stringのバリデーションはPhase 5以降で実装済みの前提)
      if let Some(ref color) = embed_color {
        if !color.is_empty() {
          EmbedColor::from_hex_string(color)
            .map_err(|_| AppError::InvalidInput("embed_colorの形式が不正です".to_string()))?;
        }
      }

      let ciphertext = encrypt_webhook_url(webhook_url, group_id, webhook_enc_key)?;

      discord_repo
        .upsert(
          group_id,
          &NotifyDiscordConfigRow {
            webhook_url_ciphertext: ciphertext,
            embed_color,
            mention_enabled,
            mention_targets,
          },
        )
        .await?;
    }
    GroupConfigData::Slack {
      webhook_url,
      mention_enabled,
      mention_targets,
    } => {
      let ciphertext = encrypt_webhook_url(webhook_url, group_id, webhook_enc_key)?;

      slack_repo
        .upsert(
          group_id,
          &NotifySlackConfigRow {
            webhook_url_ciphertext: ciphertext,
            mention_enabled,
            mention_targets,
          },
        )
        .await?;
    }
  }

  Ok(())
}

fn decrypt_webhook_url(
  ciphertext: Option<String>,
  group_id: GroupId,
  webhook_enc_key: &[u8],
) -> Result<Option<String>, AppError> {
  match ciphertext {
    None => Ok(None),
    Some(ciphertext) => {
      let encrypted = Encrypted::<WebhookUrlTag>::from_ciphertext(ciphertext);
      let aad = group_id.as_uuid().as_bytes();
      let plain = encrypted
        .decrypt(webhook_enc_key, aad)
        .map_err(|_| AppError::CryptoError)?;
      Ok(Some(plain.as_str().to_string()))
    }
  }
}

fn encrypt_webhook_url(
  plain: Option<String>,
  group_id: GroupId,
  webhook_enc_key: &[u8],
) -> Result<Option<String>, AppError> {
  match plain {
    None => Ok(None),
    Some(plain) if plain.is_empty() => Ok(None), // 空文字は未設定として扱う
    Some(plain) => {
      let aad = group_id.as_uuid().as_bytes();
      let encrypted = Encrypted::<WebhookUrlTag>::encrypt(&plain, webhook_enc_key, aad)
        .map_err(|_| AppError::CryptoError)?;
      Ok(Some(encrypted.as_str().to_string()))
    }
  }
}
