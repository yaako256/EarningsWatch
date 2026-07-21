/*
backend/crates/infra/src/postgres/queries/mod.rs
トラジェクション実装時に重複が多くなってしまうため、
共通関数として定義するというやつ
*/
pub(crate) mod earnings_query;
pub(crate) mod notify_discord_config;
pub(crate) mod notify_filter;
pub(crate) mod notify_group;
pub(crate) mod notify_queue;
pub(crate) mod notify_slack_config;
