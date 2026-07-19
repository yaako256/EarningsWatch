/*
backend/crates/identity/src/macro_def.rs
identityクレートで作成するマクロの定義
*/

// ===== マクロ定義 =====
// マクロでID型を自動作成する
// #[macro_export] ← 別クレートからは使わないためコメントアウト
macro_rules! define_id_type {
  ($name:ident) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
    pub struct $name(uuid::Uuid);

    impl $name {
      pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
      }

      pub fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self(uuid)
      }

      pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.0
      }
    }

    impl Default for $name {
      fn default() -> Self {
        Self::new()
      }
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
      }
    }
  };
}

pub(crate) use define_id_type;
