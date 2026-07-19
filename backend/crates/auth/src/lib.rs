/*
backend/crates/auth/src/lib.rs
authクレート
JWT生成・検証、PasswordHasher、TokenClaims、Role等。
*/

// crates/auth/src/lib.rs
mod password;
mod refresh_token;
mod role;
mod user;

pub use password::{PasswordHashError, hash_password};
pub use refresh_token::RefreshToken;
pub use role::Role;
pub use user::User;

// JWT生成・検証(TokenClaims、PasswordHasher等)の実装本体はPhase 7で追加する。
