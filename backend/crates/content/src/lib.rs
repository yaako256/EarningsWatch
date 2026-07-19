/*
backend/crates/content/src/lib.rs
contentクレート
お知らせ板・固定ページ(pages)のドメイン型
*/

mod page;

pub use page::{Page, PageKind, PageType};
