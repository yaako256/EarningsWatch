/*
backend/crates/app/src/import/import_filters_all.rs
全体のフィルタ一括インポート
*/

// 標準ライブラリ
use std::collections::HashMap;

// 外部クレート
use chrono::Utc;

// 内部ライブラリ
use config::ImportSettings;
use identity::{FilterId, GroupId, UserId};
use repository::{
  NotifyDiscordConfigRow, NotifyFilterRepository, NotifyGroupRepository, NotifySlackConfigRow,
  RepositoryScope, UnitOfWork,
};
use subscription::{NotifyFilter, NotifyGroup, NotifyMedium};

// 自クレート
use super::validate_row::{RowOutcome, classify_row};
use super::{GroupRefData, ImportErrorRowData, ImportFiltersResult, ImportWarningData};
use crate::AppError;

pub struct ImportFilterRowInput {
  pub ticker: String,
  pub company_name: String,
  pub group_name: String,
  pub notes: Option<String>,
  pub enabled: Option<bool>,
}

/// 全体一括設定(POST /api/filters/import)。
/// dry_run=trueの場合、DBへの反映は一切行わずプレビュー結果のみ返す。
pub async fn import_filters_all(
  group_repo: &dyn NotifyGroupRepository,
  filter_repo: &dyn NotifyFilterRepository,
  unit_of_work: &dyn UnitOfWork,
  import_settings: &ImportSettings,
  user_id: UserId,
  rows: Vec<ImportFilterRowInput>,
  dry_run: bool,
) -> Result<ImportFiltersResult, AppError> {
  let mut skipped_empty_rows = 0u32;
  let mut duplicate_count = 0u32;
  let mut error_rows = Vec::new();
  let mut warnings = Vec::new();

  // group_name -> (このグループに属する有効フィルタ一覧)
  let mut filters_by_group_name: HashMap<String, Vec<NotifyFilter>> = HashMap::new();
  // 重複検知用: (group_name, ticker)の組
  let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();

  for (idx, input) in rows.into_iter().enumerate() {
    let row_number = (idx + 1) as u32;

    match classify_row(
      &input.ticker,
      &input.company_name,
      input.notes.as_deref(),
      input.enabled,
      Some(&input.group_name),
      import_settings,
    ) {
      RowOutcome::EmptySkip => skipped_empty_rows += 1,
      RowOutcome::Broken { reason } => error_rows.push(ImportErrorRowData { row_number, reason }),
      RowOutcome::Valid {
        row,
        warnings: row_warnings,
      } => {
        let group_name = input.group_name.trim().to_string();
        let key = (group_name.clone(), row.ticker.clone());

        if !seen.insert(key) {
          duplicate_count += 1;
          continue;
        }

        for w in row_warnings {
          warnings.push(ImportWarningData {
            row_number,
            message: w,
          });
        }

        filters_by_group_name
          .entry(group_name)
          .or_default()
          .push(NotifyFilter {
            id: FilterId::new(),
            group_id: GroupId::new(), // 後でgroup_name解決後の実IDに差し替える
            ticker: row.ticker,
            company_name: row.company_name,
            notes: row.notes,
            enabled: row.enabled,
            created_at: Utc::now(),
          });
      }
    }
  }

  // グループ名解決: 既存グループの取得、新規グループの判定
  let existing_groups = group_repo.list_by_user_id(user_id).await?;
  let existing_by_name: HashMap<String, NotifyGroup> = existing_groups
    .iter()
    .cloned()
    .map(|g| (g.name.clone(), g))
    .collect();

  let mut created_groups = Vec::new();
  let mut resolved: HashMap<String, GroupId> = HashMap::new();

  for group_name in filters_by_group_name.keys() {
    if let Some(existing) = existing_by_name.get(group_name) {
      resolved.insert(group_name.clone(), existing.id);
    } else {
      // 新規グループ(本書3.4節: mediumはDiscord固定)
      let new_group_id = GroupId::new();
      resolved.insert(group_name.clone(), new_group_id);
      created_groups.push(GroupRefData {
        id: new_group_id,
        name: group_name.clone(),
      });
    }
  }

  // 今回のCSVに1件も含まれなかった既存グループ → 無効化対象
  let mentioned_names: std::collections::HashSet<&String> = filters_by_group_name.keys().collect();
  let paused_groups: Vec<GroupRefData> = existing_groups
    .iter()
    .filter(|g| !mentioned_names.contains(&g.name) && g.paused_at.is_none())
    .map(|g| GroupRefData {
      id: g.id,
      name: g.name.clone(),
    })
    .collect();

  let imported_count: u32 = filters_by_group_name.values().map(|v| v.len() as u32).sum();

  if dry_run {
    // DBへの反映を行わずプレビュー結果のみ返す
    return Ok(ImportFiltersResult {
      imported_count,
      skipped_empty_rows,
      duplicate_count,
      error_rows,
      created_groups,
      paused_groups,
      warnings,
    });
  }

  // ここから実反映(dry_run=false)
  for (group_name, mut filters) in filters_by_group_name {
    let group_id = resolved[&group_name];
    for f in filters.iter_mut() {
      f.group_id = group_id;
    }

    let is_new_group = created_groups.iter().any(|g| g.id == group_id);

    if is_new_group {
      // グループ+Discord設定+Slack設定をアトミックに作成
      let now = Utc::now();
      let new_group = NotifyGroup {
        id: group_id,
        user_id,
        name: group_name,
        medium: NotifyMedium::Discord, // 本書3.4節
        paused_at: None,
        created_at: now,
        updated_at: now,
      };

      unit_of_work
        .execute(move |scope: &mut dyn RepositoryScope| {
          Box::pin(async move {
            scope.notify_group_repository().insert(&new_group).await?;
            scope
              .notify_discord_config_repository()
              .upsert(
                new_group.id,
                &NotifyDiscordConfigRow {
                  webhook_url_ciphertext: None,
                  embed_color: None,
                  mention_enabled: false,
                  mention_targets: vec![],
                },
              )
              .await?;
            scope
              .notify_slack_config_repository()
              .upsert(
                new_group.id,
                &NotifySlackConfigRow {
                  webhook_url_ciphertext: None,
                  mention_enabled: false,
                  mention_targets: vec![],
                },
              )
              .await?;
            Ok(())
          })
        })
        .await?;
    }

    // グループ単位で完全置き換え(Phase 4・5、CSVがそのグループの完全な現在値を表す)
    filter_repo
      .replace_all_for_group(group_id, &filters)
      .await?;
  }

  for paused in &paused_groups {
    if let Some(mut group) = group_repo.find_by_id(paused.id).await? {
      group.paused_at = Some(Utc::now());
      group.updated_at = Utc::now();
      group_repo.update(&group).await?;
    }
  }

  Ok(ImportFiltersResult {
    imported_count,
    skipped_empty_rows,
    duplicate_count,
    error_rows,
    created_groups,
    paused_groups,
    warnings,
  })
}
