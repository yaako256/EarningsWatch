/*
backend/crates/auth/src/role.rs
ユーザのロールを定義
*/

// 外部クレート
use serde::{Deserialize, Serialize};

/// ロールの列挙型
// DB: users.role TEXT('admin'|'user')
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum Role {
  Admin,
  User,
}

impl Role {
  pub fn is_admin(&self) -> bool {
    matches!(self, Self::Admin)
  }
}
