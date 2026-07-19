/*
backend/crates/logging/src/sql_layer.rs
SQLレイヤーのロギング定義
*/

// 標準ライブラリ
use std::sync::Arc;
use tokio::sync::mpsc;

// 外部ライブラリ
use async_trait::async_trait;
use chrono::Utc;
use tracing::Subscriber;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

// 自クレート
use crate::entry::{LogEvent, LogLevel, LogProcess};
use crate::visit::JsonVisitor;

/// SqlLayerの書き込み先を抽象化するトレイト。
/// Phase 2時点ではPostgreSQLが存在しないため、
/// ConsoleSink(仮実装)を差し込み、Phase 3/5でPgSinkに差し替える。
#[async_trait]
pub trait LogSink: Send + Sync + 'static {
  async fn write_batch(&self, entries: &[LogEvent]);
}

/// Phase 2時点の仮実装。標準出力へそのまま書き出す。
pub struct ConsoleSink;

#[async_trait]
impl LogSink for ConsoleSink {
  async fn write_batch(&self, entries: &[LogEvent]) {
    for entry in entries {
      println!(
        "[{:?}][{:?}] {} - {:?} {}",
        entry.process, entry.level, entry.target, entry.message, entry.fields
      );
    }
  }
}

// n件溜まったら。将来configへ切り出す
const BATCH_FLUSH_THRESHOLD: usize = 50;

enum WriterMessage {
  Event(LogEvent),
  FlushNow, // server: フロントからログ表示リクエストが来たら / cli: 単発実行終了時
}

pub struct SqlLayer {
  sender: mpsc::UnboundedSender<WriterMessage>,
  process: LogProcess,
}

impl SqlLayer {
  /// SqlLayer本体と、バックグラウンドのバッチ書き込みタスクの JoinHandle を返す。
  /// タスクはserver/cliプロセスの生存期間中ずっと動き続ける。
  pub fn new(process: LogProcess, sink: impl LogSink) -> (Self, tokio::task::JoinHandle<()>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let handle = tokio::spawn(batch_writer_task(rx, Arc::new(sink)));
    (
      Self {
        sender: tx,
        process,
      },
      handle,
    )
  }

  /// server: フロントエンドからログ表示のリクエストが来た際に呼ぶ想定(design 1.2章)
  /// cli: monitor/notify単発実行の終了時に呼ぶ想定(design 1.2章、最終flush)
  pub fn flush_now(&self) {
    let _ = self.sender.send(WriterMessage::FlushNow);
  }
}

async fn batch_writer_task(mut rx: mpsc::UnboundedReceiver<WriterMessage>, sink: Arc<dyn LogSink>) {
  let mut buffer: Vec<LogEvent> = Vec::new();

  while let Some(msg) = rx.recv().await {
    match msg {
      WriterMessage::Event(event) => {
        buffer.push(event);
        if buffer.len() >= BATCH_FLUSH_THRESHOLD {
          sink.write_batch(&buffer).await;
          buffer.clear();
        }
      }
      WriterMessage::FlushNow => {
        if !buffer.is_empty() {
          sink.write_batch(&buffer).await;
          buffer.clear();
        }
      }
    }
  }

  // チャネルが閉じた(プロセス終了)場合の最終flush
  if !buffer.is_empty() {
    sink.write_batch(&buffer).await;
  }
}

impl<S: Subscriber> Layer<S> for SqlLayer {
  fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
    let mut visitor = JsonVisitor::default();
    event.record(&mut visitor);

    let log_event = LogEvent {
      timestamp: Utc::now(),
      level: LogLevel::from_tracing_level(event.metadata().level()),
      process: self.process,
      target: event.metadata().target().to_string(),
      message: visitor.message,
      fields: serde_json::Value::Object(visitor.fields),
    };

    // on_eventは同期関数のため.awaitできない。チャネル送信のみ行う非ブロッキング設計
    // (design/03-features/admin-dashboard.md 1.2章)
    let _ = self.sender.send(WriterMessage::Event(log_event));
  }
}
