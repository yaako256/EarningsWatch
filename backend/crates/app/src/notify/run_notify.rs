/*
backend/crates/app/src/notify/run_notify.rs
送信処理を実行するユースケース
*/

// 標準ライブラリ
use std::time::{Duration, Instant};

// 外部クレート
use chrono::Utc;

// 内部ライブラリ
use config::RetrySettings;
use crypto::{Encrypted, WebhookUrlTag};
use identity::GroupId;
use notifier::discord::{DiscordMessageInput, EmbedColor, send_discord_message};
use repository::{
  NotifyDiscordConfigRepository, NotifyFilterRepository, NotifyGroupRepository,
  NotifyHistoryRepository, NotifyQueueRepository, SystemRunRepository,
};
use subscription::{NotifyHistoryEntry, NotifyMedium, NotifyQueueEntry, NotifyStatus};

// 自クレート
use crate::AppError;

pub struct NotifyRunResult {
  pub total_send_count: u32,
  pub success_send_count: u32,
  pub duration_ms: i32,
}

#[allow(clippy::too_many_arguments)]
pub async fn run_notify(
  queue_repo: &dyn NotifyQueueRepository,
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  discord_config_repo: &dyn NotifyDiscordConfigRepository,
  history_repo: &dyn NotifyHistoryRepository,
  system_run_repo: &dyn SystemRunRepository,
  webhook_enc_key: &[u8],
  retry_settings: &RetrySettings,
) -> Result<NotifyRunResult, AppError> {
  let run_at = Utc::now();
  let start = Instant::now();

  // 1. monitor健全性チェック(design/01-db-schema.md 6章、リトライ込み、本書3.6節)
  let mut healthy = false;
  for attempt in 0..retry_settings.monitor_health_check_retries {
    if !queue_repo.monitor_marker_exists().await? {
      healthy = true;
      break;
    }
    tracing::warn!(
      attempt,
      "monitorマーカー行が残っています。再チェックまで待機します。"
    );
    if attempt + 1 < retry_settings.monitor_health_check_retries {
      tokio::time::sleep(Duration::from_secs(
        retry_settings.monitor_health_check_interval_seconds,
      ))
      .await;
    }
  }

  if !healthy {
    // MemoryLayer経由の警告通知(12章でMemoryLayerを登録する前提。ここではtracing::error!を発行するのみ)
    tracing::error!(
      "monitorが完了していない可能性があります(マーカー行が残存)。notifyを中断します。"
    );
    return Err(AppError::MonitorNotHealthy);
  }

  // 2. 送信対象の決算データ行を取得
  let ready_entries = queue_repo.list_ready().await?;

  // 3. 全グループを順に処理する(design/03-features/notification.md 9章)
  let groups = group_repo.list_all().await?;

  let mut total_send_count = 0u32;
  let mut success_send_count = 0u32;

  for group in groups.iter().filter(|g| g.paused_at.is_none()) {
    if group.medium != NotifyMedium::Discord {
      continue; // Slackは仮実装のため対象外(引き継ぎメモ参照)
    }

    let (total, success) = process_group(
      group.id,
      filter_repo,
      discord_config_repo,
      history_repo,
      &ready_entries,
      webhook_enc_key,
      retry_settings,
    )
    .await?;

    total_send_count += total;
    success_send_count += success;
  }

  // 4. notify_queueのステータス更新(本書3.5節: 処理済みは常にsent)
  for entry in &ready_entries {
    queue_repo
      .update_status(entry.id, NotifyStatus::Sent)
      .await?;
  }

  let duration_ms = start.elapsed().as_millis() as i32;
  system_run_repo
    .record_notify_run(
      run_at,
      duration_ms,
      total_send_count as i32,
      success_send_count as i32,
    )
    .await?;

  tracing::info!(
    total_send_count,
    success_send_count,
    duration_ms,
    "notify completed"
  );

  Ok(NotifyRunResult {
    total_send_count,
    success_send_count,
    duration_ms,
  })
}

#[allow(clippy::too_many_arguments)]
async fn process_group(
  group_id: GroupId,
  filter_repo: &dyn NotifyFilterRepository,
  discord_config_repo: &dyn NotifyDiscordConfigRepository,
  history_repo: &dyn NotifyHistoryRepository,
  ready_entries: &[NotifyQueueEntry],
  webhook_enc_key: &[u8],
  retry_settings: &RetrySettings,
) -> Result<(u32, u32), AppError> {
  let filters = filter_repo.list_by_group_id(group_id).await?;
  let enabled_filters: Vec<_> = filters.into_iter().filter(|f| f.enabled).collect();
  if enabled_filters.is_empty() {
    return Ok((0, 0));
  }

  let Some(discord_config) = discord_config_repo.find_by_group_id(group_id).await? else {
    return Ok((0, 0));
  };
  let Some(ciphertext) = discord_config.webhook_url_ciphertext.clone() else {
    return Ok((0, 0)); // webhook未設定のグループはスキップ
  };

  let aad = group_id.as_uuid();
  let webhook_url = match Encrypted::<WebhookUrlTag>::from_ciphertext(ciphertext)
    .decrypt(webhook_enc_key, aad.as_bytes())
  {
    Ok(plain) => plain.as_str().to_string(),
    Err(_) => {
      tracing::warn!(%group_id, "webhook_urlの復号に失敗したためグループをスキップしました");
      return Ok((0, 0));
    }
  };

  let embed_color = discord_config
    .embed_color
    .as_deref()
    .and_then(|s| EmbedColor::from_hex_string(s).ok())
    .unwrap_or(EmbedColor::DEFAULT);

  let mut total = 0u32;
  let mut success = 0u32;

  for entry in ready_entries {
    // design/03-features/notification.md 2章: 証券コード・銘柄名の片方一致で対象に含める
    let matched = enabled_filters
      .iter()
      .any(|f| f.ticker == entry.ticker || f.company_name == entry.company_name);
    if !matched {
      continue;
    }

    total += 1;

    let input = DiscordMessageInput {
      ticker: &entry.ticker,
      company_name: &entry.company_name,
      title: &entry.title,
      summary: &entry.summary,
      url: &entry.url,
      evaluation_label: &format!("{:?}", entry.evaluation),
      embed_color,
      mention_targets: &discord_config.mention_targets,
    };

    // design/03-features/notification.md 10章: 個別送信失敗時のリトライ(当該notify実行内で完結)
    let mut sent = false;
    for attempt in 0..retry_settings.send_retry_count {
      match send_discord_message(&webhook_url, &input).await {
        Ok(()) => {
          sent = true;
          break;
        }
        Err(e) => {
          tracing::warn!(%group_id, attempt, error = %e, "Discordへの送信に失敗しました");
          if attempt + 1 < retry_settings.send_retry_count {
            tokio::time::sleep(Duration::from_secs(
              retry_settings.send_retry_interval_seconds,
            ))
            .await;
          }
        }
      }
    }

    if sent {
      success += 1;
    }

    history_repo
      .insert(&NotifyHistoryEntry {
        id: 0, // DB側で自動採番(9章参照)
        group_id: Some(group_id),
        fingerprint: entry.fingerprint.clone(),
        sent_at: Utc::now(),
        status: if sent {
          NotifyStatus::Sent
        } else {
          NotifyStatus::Failed
        },
      })
      .await?;
  }

  Ok((total, success))
}
