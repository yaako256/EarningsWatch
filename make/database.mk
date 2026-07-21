# make/database.mk
# ==================================================
### Database
# ==================================================

.PHONY: migrate


# ==================================================
# Migration
# ==================================================

## migrationの実行
# dev:
#   cargo run -p cli -- migrate
# prod:
#   /app/yaakodrive-cli migrate
# migrate:
# 	$(MAKE) cli \
# 		ARGS="migrate --migrations-path $(MIGRATIONS_PATH)"


# ==================================================
# Database Viewer
# ==================================================
.PHONY: \
	db-logs db-logs-x \
	db-users db-users-x \
	db-refresh-tokens db-refresh-tokens-x \
	db-user-settings db-user-settings-x \
	db-notify-groups db-notify-groups-x \
	db-notify-discord-configs db-notify-discord-configs-x \
	db-notify-slack-configs db-notify-slack-configs-x \
	db-notify-filters db-notify-filters-x \
	db-earnings db-earnings-x \
	db-notify-queue db-notify-queue-x \
	db-notify-history db-notify-history-x \
	db-system-runs db-system-runs-x \
	db-system-notify-config db-system-notify-config-x \
	db-pages db-pages-x

## logsテーブル(一部)
db-logs:
	$(PSQL) -c "SELECT id, timestamp, level, process, target, message FROM logs;"

## logsテーブル(全件)
db-logs-x:
	$(PSQL) -x -c "SELECT * FROM logs;"

## usersテーブル(一部)
db-users:
	$(PSQL) -c "SELECT id, username, role, created_at, updated_at, disabled_at FROM users;"

## usersテーブル(全件)
db-users-x:
	$(PSQL) -x -c "SELECT * FROM users;"

## refresh_tokensテーブル(一部)
db-refresh-tokens:
	$(PSQL) -c "SELECT id, user_id, user_agent, expires_at, created_at, revoked_at FROM refresh_tokens;"

## refresh_tokensテーブル(全件)
db-refresh-tokens-x:
	$(PSQL) -x -c "SELECT * FROM refresh_tokens;"

## user_settingsテーブル(一部)
db-user-settings:
	$(PSQL) -c "SELECT user_id, memo, updated_at FROM user_settings;"

## user_settingsテーブル(全件)
db-user-settings-x:
	$(PSQL) -x -c "SELECT * FROM user_settings;"

## notify_groupsテーブル(一部)
db-notify-groups:
	$(PSQL) -c "SELECT id, user_id, name, medium, paused_at, created_at, updated_at FROM notify_groups;"

## notify_groupsテーブル(全件)
db-notify-groups-x:
	$(PSQL) -x -c "SELECT * FROM notify_groups;"

## notify_discord_configsテーブル(一部)
db-notify-discord-configs:
	$(PSQL) -c "SELECT group_id, webhook_url, embed_color, mention_enabled, mention_targets FROM notify_discord_configs;"

## notify_discord_configsテーブル(全件)
db-notify-discord-configs-x:
	$(PSQL) -x -c "SELECT * FROM notify_discord_configs;"

## notify_slack_configsテーブル(一部)
db-notify-slack-configs:
	$(PSQL) -c "SELECT group_id, webhook_url, mention_enabled, mention_targets FROM notify_slack_configs;"

## notify_slack_configsテーブル(全件)
db-notify-slack-configs-x:
	$(PSQL) -x -c "SELECT * FROM notify_slack_configs;"

## notify_filtersテーブル(一部)
db-notify-filters:
	$(PSQL) -c "SELECT id, group_id, ticker, company_name, enabled, created_at FROM notify_filters;"

## notify_filtersテーブル(全件)
db-notify-filters-x:
	$(PSQL) -x -c "SELECT * FROM notify_filters;"

## earningsテーブル(一部)
db-earnings:
	$(PSQL) -c "SELECT id, ticker, company_name, published_at, title, evaluation, source FROM earnings;"

## earningsテーブル(全件)
db-earnings-x:
	$(PSQL) -x -c "SELECT * FROM earnings;"

## notify_queueテーブル(一部)
db-notify-queue:
	$(PSQL) -c "SELECT id, fingerprint, is_monitor_marker, source, fetched_at, ticker, company_name, published_at, title, status FROM notify_queue;"

## notify_queueテーブル(全件)
db-notify-queue-x:
	$(PSQL) -x -c "SELECT * FROM notify_queue;"

## notify_historyテーブル(一部)
db-notify-history:
	$(PSQL) -c "SELECT id, group_id, fingerprint, sent_at, status FROM notify_history;"

## notify_historyテーブル(全件)
db-notify-history-x:
	$(PSQL) -x -c "SELECT * FROM notify_history;"

## system_runsテーブル(一部)
db-system-runs:
	$(PSQL) -c "SELECT id, run_type, run_at, duration_ms, new_earnings_count, total_send_count, success_send_count FROM system_runs;"

## system_runsテーブル(全件)
db-system-runs-x:
	$(PSQL) -x -c "SELECT * FROM system_runs;"

## system_notify_configテーブル(一部)
db-system-notify-config:
	$(PSQL) -c "SELECT id, medium, webhook_url, mention_enabled, mention_targets, updated_at FROM system_notify_config;"

## system_notify_configテーブル(全件)
db-system-notify-config-x:
	$(PSQL) -x -c "SELECT * FROM system_notify_config;"

## pagesテーブル(一部)
db-pages:
	$(PSQL) -c "SELECT id, type, title, display_order, is_published, created_at, updated_at, created_by FROM pages;"

## pagesテーブル(全件)
db-pages-x:
	$(PSQL) -x -c "SELECT * FROM pages;"


# ==================================================
# Database appendix Viewer
# ==================================================
.PHONY: \
	db-check-notify-group-create

# グループ作成時の送信媒体ごとの固有設定テーブルとの結びつき確認
db-check-notify-group-create:
	@$(PSQL) -x -c "\
	SELECT \
		g.name, \
		d.group_id IS NOT NULL AS has_discord, \
		s.group_id IS NOT NULL AS has_slack \
	FROM notify_groups g \
	LEFT JOIN notify_discord_configs d ON d.group_id = g.id \
	LEFT JOIN notify_slack_configs s ON s.group_id = g.id \
	ORDER BY g.created_at DESC \
	LIMIT 10;"