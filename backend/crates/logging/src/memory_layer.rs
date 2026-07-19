/*
backend/crates/logging/src/memory_layer.rs
メモリレイヤーのロギング定義
*/

// 標準ライブラリ
use std::sync::{Arc, Mutex};

// 外部クレート
use async_trait::async_trait;
use chrono::Utc;
use tracing::Subscriber;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

// 自クレート
use crate::entry::{LogEvent, LogLevel, LogProcess};
use crate::visit::JsonVisitor;

/// warn/error以上のログをまとめて送る通知先を抽象化するトレイト。
/// Phase 2時点ではnotifierクレートのDiscord実装がまだないため、
/// ConsoleWarnNotifySink(仮実装)を差し込み、Phase 11以降で実送信に差し替える。
#[async_trait]
pub trait WarnNotifySink: Send + Sync + 'static {
  async fn notify(&self, process: LogProcess, entries: &[LogEvent]);
}

/// Phase 2時点の仮実装。標準出力へ警告として書き出す。
pub struct ConsoleWarnNotifySink;

#[async_trait]
impl WarnNotifySink for ConsoleWarnNotifySink {
  async fn notify(&self, process: LogProcess, entries: &[LogEvent]) {
    eprintln!(
      "[MemoryLayer][{process:?}] {}件の警告/エラーを検知(仮実装:コンソール出力のみ)",
      entries.len()
    );
    for entry in entries {
      eprintln!(
        "  - [{:?}] {}: {:?}",
        entry.level, entry.target, entry.message
      );
    }
  }
}

pub struct MemoryLayer {
  process: LogProcess,
  buffer: Arc<Mutex<Vec<LogEvent>>>,
  sink: Arc<dyn WarnNotifySink>,
}

impl MemoryLayer {
  pub fn new(process: LogProcess, sink: impl WarnNotifySink) -> Self {
    Self {
      process,
      buffer: Arc::new(Mutex::new(Vec::new())),
      sink: Arc::new(sink),
    }
  }

  /// flushトリガーは呼び出し側(cli/server)が注入する(design/03-features/admin-dashboard.md 1.4章)。
  /// - cli(monitor/notify): プロセス終了時に1回呼ぶ
  /// - server: 1分程度(config: logging.server_flush_window_seconds)の周期タイマーで呼ぶ
  pub async fn flush(&self) {
    let entries = {
      let mut buf = self.buffer.lock().expect("MemoryLayer buffer poisoned");
      std::mem::take(&mut *buf)
    };
    if !entries.is_empty() {
      self.sink.notify(self.process, &entries).await;
    }
  }
}

impl<S: Subscriber> Layer<S> for MemoryLayer {
  fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
    let level = LogLevel::from_tracing_level(event.metadata().level());
    if !level.is_warn_or_above() {
      return;
    }

    let mut visitor = JsonVisitor::default();
    event.record(&mut visitor);

    let log_event = LogEvent {
      timestamp: Utc::now(),
      level,
      process: self.process,
      target: event.metadata().target().to_string(),
      message: visitor.message,
      fields: serde_json::Value::Object(visitor.fields),
    };

    let mut buf = self.buffer.lock().expect("MemoryLayer buffer poisoned");
    buf.push(log_event);
  }
}
