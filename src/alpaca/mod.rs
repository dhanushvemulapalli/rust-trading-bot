//! Alpaca module — broker-specific types and networking.
//!
//! Nothing outside this module should import Alpaca-specific types directly.
//! Convert Alpaca responses into domain types before returning them.

pub mod account;
pub mod client;
pub mod market_data;
pub mod orders;
