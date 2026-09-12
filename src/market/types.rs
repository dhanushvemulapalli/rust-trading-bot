//! Core domain market data types.

use serde::{Deserialize, Serialize};

/// An OHLCV bar for one symbol over one time period.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bar {
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    /// RFC 3339 timestamp of the bar's close.
    pub timestamp: String,
}

/// A single trade (tick).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub symbol: String,
    pub price: f64,
    pub size: u32,
    pub timestamp: String,
}

/// A bid/ask quote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub symbol: String,
    pub bid: f64,
    pub bid_size: u32,
    pub ask: f64,
    pub ask_size: u32,
    pub timestamp: String,
}

/// Direction of a position or order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Long,
    Short,
}

/// Current position in a symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub side: Side,
    pub qty: f64,
    pub avg_entry_price: f64,
    pub market_value: f64,
    pub unrealized_pl: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_round_trips_json() {
        let bar = Bar {
            symbol: "AAPL".into(),
            open: 100.0,
            high: 105.0,
            low: 99.0,
            close: 103.5,
            volume: 50_000,
            timestamp: "2024-01-15T09:30:00Z".into(),
        };
        let json = serde_json::to_string(&bar).unwrap();
        let decoded: Bar = serde_json::from_str(&json).unwrap();
        assert_eq!(bar, decoded);
    }
}
