/*
backend/crates/earnings/src/lib.rs
earningsクレート
決算情報のコアドメインを定義
*/

mod evaluation;
mod fingerprint;
mod record;
mod ticker;

pub use evaluation::{EarningsEvaluation, EarningsSource};
pub use fingerprint::compute_fingerprint;
pub use record::{Earnings, EarningsRecord, MonitoredEarningsReport};
pub use ticker::normalize_ticker;
