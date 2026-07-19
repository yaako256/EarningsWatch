/*
backend/crates/identity/src/lib.rs
identityクレート
各種ID型をマクロを使って定義する(Dry原則)。
*/

mod macro_def;
use macro_def::define_id_type;

// ===== ID型定義 =====
define_id_type!(UserId);
define_id_type!(GroupId);
define_id_type!(FilterId);
define_id_type!(PageId);
define_id_type!(RefreshTokenId);
