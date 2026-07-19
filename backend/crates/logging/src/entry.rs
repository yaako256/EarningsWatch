/*
backend/crates/logging/src/entry.rs
ログの型定義など
*/

// 外部クレート
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

// DB: logs.level VARCHAR(5) CHECK(...)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "UPPERCASE")]
pub enum LogLevel {
  Trace,
  Debug,
  Info,
  Warn,
  Error,
}

impl LogLevel {
  pub fn from_tracing_level(level: &tracing::Level) -> Self {
    match *level {
      tracing::Level::TRACE => Self::Trace,
      tracing::Level::DEBUG => Self::Debug,
      tracing::Level::INFO => Self::Info,
      tracing::Level::WARN => Self::Warn,
      tracing::Level::ERROR => Self::Error,
    }
  }

  /// MemoryLayerの「warn/error以上」判定に使う
  pub fn is_warn_or_above(&self) -> bool {
    matches!(self, Self::Warn | Self::Error)
  }
}

/// どこのログかの列挙
// DB: logs.process log_process enum('server'|'monitor'|'notify')
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "log_process", rename_all = "lowercase")]
pub enum LogProcess {
  Server,
  Monitor,
  Notify,
}

/// DB挿入前(idがまだ存在しない)のログイベント。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
  pub timestamp: DateTime<Utc>,
  pub level: LogLevel,
  pub process: LogProcess,
  pub target: String,
  pub message: Option<String>,
  pub fields: JsonValue,
}

/// DB: logsテーブル1行分の型定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
  pub id: i64,
  pub timestamp: DateTime<Utc>,
  pub level: LogLevel,
  pub process: LogProcess,
  pub target: String,
  pub message: Option<String>,
  pub fields: JsonValue,
}
