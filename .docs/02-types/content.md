# EarningsWatch 本設計書 02-types. contentクレート(新設)

> `仮設計書-型定義.md` 7章を元にしている。7.4節の残課題(`display_order`の一意性制約の要否)は`01-db-schema.md` 10章で「一意性制約なし」として確定済みのため、その旨を反映した。それ以外は内容の変更なく、構成の移設のみ。

## 目次
1. [経緯・方針](#1-経緯方針)
2. [型定義](#2-型定義)
3. [対応関係](#3-対応関係)
4. [残課題](#4-残課題次節以降で検討)

---

## 1. 経緯・方針

お知らせ板・固定ページ(新設機能)の型定義中に、決算・通知いずれの既存クレートとも性質が異なる「コンテンツ管理」の型が必要になったため、新規クレートとして切り出した。

- 依存関係(`00-overview.md` 6.4節への追記): `content -> identity`のみ
- `PageId`は`02-types/identity.md`で先行して定義済み

## 2. 型定義

```rust
use chrono::{DateTime, Utc};
use identity::{PageId, UserId};
use serde::{Deserialize, Serialize};

// DB: pages.type page_type enum('blog'|'static')(専用enum型として新設、01-db-schema.md 10章)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "page_type", rename_all = "lowercase")]
pub enum PageType {
    Blog,
    Static,
}

// display_orderはstatic限定・blogでは常にNULLという「typeによってフィールドの意味が変わる」構造のため、
// 判別Union(PageKind)で表現する
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageKind {
    Blog,
    Static { display_order: i32 },
}

impl PageKind {
    pub fn page_type(&self) -> PageType {
        match self {
            Self::Blog => PageType::Blog,
            Self::Static { .. } => PageType::Static,
        }
    }
}

// DB: pagesテーブルの1行に対応(01-db-schema.md 10章)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: PageId,
    pub kind: PageKind,
    pub title: String,
    pub content_markdown: String,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: UserId,
}
```

## 3. 対応関係

| 型 | 対応セクション | 備考 |
|---|---|---|
| `PageType` | `01-db-schema.md` 10章 | 専用enum型(`page_type`)としてDDLに新設 |
| `PageKind` | 同上 | 判別Union。`display_order`は`Static`のみ持つ |
| `Page` | 同上 | `pages`テーブルの1行に対応 |

## 4. 残課題(次節以降で検討)

- `display_order`の一意性制約は設けない方針で確定済み(`01-db-schema.md` 10章)。並べ替え時の一時的な重複はUI側で吸収する
- 並べ替えAPI(`PATCH /api/pages/{id}/order`、`02-types/api.md` 11章)の具体的な再計算アルゴリズムは`03-features/notice-board.md`で検討する
