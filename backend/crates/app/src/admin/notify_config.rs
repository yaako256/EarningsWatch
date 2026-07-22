/*
backend/crates/app/src/admin/notify_config.rs
システム通知の設定をするユースケース
*/

// 内部ライブラリ
use crypto::{Encrypted, SystemNotifyWebhookUrlTag};
use repository::SystemNotifyConfigRepository;
use subscription::{NotifyMedium, SystemNotifyConfig};

// 自クレート
use crate::AppError;

pub struct NotifyConfigData {
  pub medium: NotifyMedium,
  pub webhook_url: Option<String>, // 復号済み平文
  pub mention_enabled: bool,
  pub mention_targets: Vec<String>,
}

const SYSTEM_NOTIFY_CONFIG_AAD: &[u8] = b"system_notify_config"; // 固定AAD(グループ単位のような識別子がないため)

pub async fn get_notify_config(
  config_repo: &dyn SystemNotifyConfigRepository,
  webhook_enc_key: &[u8],
) -> Result<Option<NotifyConfigData>, AppError> {
  let Some(config) = config_repo.get().await? else {
    return Ok(None);
  };

  let webhook_url = match config.webhook_url {
    Some(encrypted) => {
      let plain = encrypted
        .decrypt(webhook_enc_key, SYSTEM_NOTIFY_CONFIG_AAD)
        .map_err(|_| AppError::CryptoError)?;
      Some(plain.as_str().to_string())
    }
    None => None,
  };

  Ok(Some(NotifyConfigData {
    medium: config.medium,
    webhook_url,
    mention_enabled: config.mention_enabled,
    mention_targets: config.mention_targets,
  }))
}

pub async fn update_notify_config(
  config_repo: &dyn SystemNotifyConfigRepository,
  data: NotifyConfigData,
  webhook_enc_key: &[u8],
) -> Result<(), AppError> {
  let webhook_url = match data.webhook_url {
    Some(plain) if !plain.is_empty() => Some(
      Encrypted::<SystemNotifyWebhookUrlTag>::encrypt(
        &plain,
        webhook_enc_key,
        SYSTEM_NOTIFY_CONFIG_AAD,
      )
      .map_err(|_| AppError::CryptoError)?,
    ),
    _ => None,
  };

  config_repo
    .upsert(&SystemNotifyConfig {
      medium: data.medium,
      webhook_url,
      mention_enabled: data.mention_enabled,
      mention_targets: data.mention_targets,
      updated_at: chrono::Utc::now(),
    })
    .await?;

  Ok(())
}
