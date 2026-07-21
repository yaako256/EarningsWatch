// backend/crates/infra/src/postgres/queries/notify_slack_config.rs

use identity::GroupId;
use repository::{NotifySlackConfigRow, RepositoryError};
use sqlx::{Executor, Postgres};

use crate::error_mapping::map_error;

pub(crate) async fn find_by_group_id<'e, E>(
  executor: E,
  group_id: GroupId,
) -> Result<Option<NotifySlackConfigRow>, RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
  let row = sqlx::query!(
    r#"
    SELECT webhook_url, mention_enabled, mention_targets
    FROM notify_slack_configs WHERE group_id = $1
    "#,
    group_id.as_uuid()
  )
  .fetch_optional(executor)
  .await
  .map_err(map_error)?;

  Ok(row.map(|r| NotifySlackConfigRow {
    webhook_url_ciphertext: r.webhook_url,
    mention_enabled: r.mention_enabled,
    mention_targets: r.mention_targets,
  }))
}

pub(crate) async fn upsert<'e, E>(
  executor: E,
  group_id: GroupId,
  row: &NotifySlackConfigRow,
) -> Result<(), RepositoryError>
where
  E: Executor<'e, Database = Postgres>,
{
  sqlx::query!(
    r#"
    INSERT INTO notify_slack_configs (group_id, webhook_url, mention_enabled, mention_targets)
    VALUES ($1, $2, $3, $4)
    ON CONFLICT (group_id) DO UPDATE
    SET webhook_url = EXCLUDED.webhook_url,
      mention_enabled = EXCLUDED.mention_enabled,
      mention_targets = EXCLUDED.mention_targets
    "#,
    group_id.as_uuid(),
    row.webhook_url_ciphertext,
    row.mention_enabled,
    &row.mention_targets
  )
  .execute(executor)
  .await
  .map_err(map_error)?;

  Ok(())
}
