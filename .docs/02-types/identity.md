# EarningsWatch 本設計書 02-types. identityクレート

> `仮設計書-型定義.md` 1章を元にしている。内容の変更はなく、構成の移設のみ。

## 目次
1. [方針](#1-方針)
2. [Cargo.toml](#2-cargotoml)
3. [ファイル構造](#3-ファイル構造)
4. [lib.rs](#4-librs)
5. [対象一覧](#5-対象一覧)
6. [DB変換方針](#6-db変換方針参考infraクレート側での実装イメージ)

---

## 1. 方針

- YaakoDriveのnewtypeパターンをそのまま踏襲する
- `identity`クレートはsqlxに依存しない(YaakoDriveと同様)。DB(`Uuid`カラム)との変換は`as_uuid()` / `from_uuid()`を介して`infra`クレート側で行う
- 型ごとに完全に同一の実装(フィールド1つ・impl4つ)が繰り返されるため、DRY原則に基づき`macro_rules!`でマクロ化する
- マクロ化により1型あたりの記述量が数行に収まるため、YaakoDriveのような1型1ファイル構成(`user_id.rs`等)は採用せず、`lib.rs`にまとめて記述する

## 2. Cargo.toml

```toml
[package]
name = "identity"
version = "0.1.0"
edition = "2024"

[dependencies]
# workspace共通外部クレート
serde = { workspace = true }
uuid = { workspace = true }
```

## 3. ファイル構造

```text
identity/
├── Cargo.toml
└── src/
    └── lib.rs
```

## 4. lib.rs

```rust
// ===== マクロ定義 =====

#[macro_export]
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

// ===== ID型定義 =====

define_id_type!(UserId);
define_id_type!(GroupId);
define_id_type!(FilterId);
define_id_type!(PageId);
define_id_type!(RefreshTokenId);
```

## 5. 対象一覧

| 型 | 対応テーブル | 対応セクション |
|---|---|---|
| `UserId` | `users` | `01-db-schema.md` 2章 |
| `GroupId` | `notify_groups` | `01-db-schema.md` 4章 |
| `FilterId` | `notify_filters` | `01-db-schema.md` 4章 |
| `PageId` | `pages`(お知らせ板・固定ページ、新設) | `01-db-schema.md` 10章 |
| `RefreshTokenId` | `refresh_tokens` | `01-db-schema.md` 3章 |

## 6. DB変換方針(参考、`infra`クレート側での実装イメージ)

`identity`クレート自体はsqlxに依存しないため、DBとの相互変換は`infra`クレート側で`as_uuid()` / `from_uuid()`を用いて行う。

```rust
// infra crate側での利用例(参考)
sqlx::query_as!(
    GroupRow,
    "SELECT id, user_id, name FROM notify_groups WHERE id = $1",
    group_id.as_uuid()  // &Uuid を渡す
)
```

```rust
// DBから取得したUuidをID型へ変換する例(参考)
let group_id = GroupId::from_uuid(row.id);
```
