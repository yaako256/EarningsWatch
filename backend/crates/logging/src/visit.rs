// crates/logging/src/visit.rs
use serde_json::{Map, Value};
use tracing::field::{Field, Visit};

/// tracingの`info!(x = 1, y = "abc", "message")`のようなフィールドを
/// message文字列と構造化フィールド(JSONB保存用)に分離して収集する。
#[derive(Default)]
pub struct JsonVisitor {
  pub message: Option<String>,
  pub fields: Map<String, Value>,
}

impl Visit for JsonVisitor {
  fn record_str(&mut self, field: &Field, value: &str) {
    if field.name() == "message" {
      self.message = Some(value.to_string());
    } else {
      self
        .fields
        .insert(field.name().to_string(), Value::String(value.to_string()));
    }
  }

  fn record_i64(&mut self, field: &Field, value: i64) {
    self
      .fields
      .insert(field.name().to_string(), Value::from(value));
  }

  fn record_u64(&mut self, field: &Field, value: u64) {
    self
      .fields
      .insert(field.name().to_string(), Value::from(value));
  }

  fn record_bool(&mut self, field: &Field, value: bool) {
    self
      .fields
      .insert(field.name().to_string(), Value::from(value));
  }

  fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
    let s = format!("{value:?}");
    if field.name() == "message" {
      self.message = Some(s);
    } else {
      self
        .fields
        .insert(field.name().to_string(), Value::String(s));
    }
  }
}
