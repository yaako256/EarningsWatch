// crates/api/src/lib.rs
//! apiクレート。HTTP層(Routing、Request解析、Response生成、Cookie処理、
//! JWT認証ミドルウェア、エンベロープ形式へのエラー変換)。
//! response.rs/handlers/等の実装本体はPhase 7以降で行う(design/02-types/api.md 1章のファイル構造参照)。

pub mod handlers;
pub mod response;
pub mod router;
pub mod state;
