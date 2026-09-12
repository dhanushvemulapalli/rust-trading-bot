//! EMA Crossover + RSI Momentum Filter Strategy.
//!
//! Generates:
//! - BUY:  EMA short crosses above EMA long AND RSI is in the 40..=70 range.
//! - SELL: EMA short crosses below EMA long AND RSI is in the 30..=60 range,
//!         OR RSI >= 80.0 (overbought take-profit).

use crate::market::{
    indicators::{ema, rsi},
    types::Bar,
};
use crate::strategy::{Signal, Strategy};
use tracing::{debug, info};

pub struct EmaRsiStrategy {
    symbol: String,
    short_period: usize,
    long_period: usize,
    rsi_period: usize,
    prices: Vec<f64>,
    prev_short_above: Option<bool>,
}

impl EmaRsiStrategy {
    pub fn new(
        symbol: impl Into<String>,
        short_period: usize,
        long_period: usize,
        rsi_period: usize,
    ) -> Self {
        assert!(short_period > 0, "short_period must be > 0");
        assert!(long_period > short_period, "long_period must be > short_period");
        assert!(rsi_period > 0, "rsi_period must be > 0");

        Self {
            symbol: symbol.into(),
            short_period,
            long_period,
            rsi_period,
            prices: Vec::new(),
            prev_short_above: None,
        }
    }
}

impl Strategy for EmaRsiStrategy {
    fn name(&self) -> &str {
        "EmaRsiStrategy"
    }

    fn on_bar(&mut self, bar: &Bar) -> Signal {
        if bar.symbol != self.symbol {
            return Signal::Hold;
        }

        self.prices.push(bar.close);

        let max_len = self.long_period * 3;
        if self.prices.len() > max_len {
            self.prices.remove(0);
        }

        let short_ema = match ema(&self.prices, self.short_period) {
            Some(v) => v,
            None => return Signal::Hold,
        };
        let long_ema = match ema(&self.prices, self.long_period) {
            Some(v) => v,
            None => return Signal::Hold,
        };
        let current_rsi = match rsi(&self.prices, self.rsi_period) {
            Some(v) => v,
            None => return Signal::Hold,
        };

        let short_above = short_ema > long_ema;

        debug!(
            symbol = %bar.symbol,
            short_ema = %format!("{:.4}", short_ema),
            long_ema = %format!("{:.4}", long_ema),
            rsi = %format!("{:.2}", current_rsi),
            "Strategy indicators"
        );

        // Emergency overbought exit check
        if current_rsi >= 80.0 && self.prev_short_above == Some(true) {
            info!(
                symbol = %bar.symbol,
                rsi = %current_rsi,
                "RSI overbought (>= 80) -> SELL take-profit exit"
            );
            self.prev_short_above = Some(short_above);
            return Signal::Sell;
        }

        let signal = match self.prev_short_above {
            None => Signal::Hold,
            Some(prev) if !prev && short_above => {
                // Bullish crossover: check RSI confirmation
                if (40.0..=70.0).contains(&current_rsi) {
                    info!(
                        symbol = %bar.symbol,
                        rsi = %current_rsi,
                        "EMA crossover UP with confirmed RSI -> BUY signal"
                    );
                    Signal::Buy
                } else {
                    debug!(
                        symbol = %bar.symbol,
                        rsi = %current_rsi,
                        "EMA crossed UP but RSI out of range -> HOLD"
                    );
                    Signal::Hold
                }
            }
            Some(prev) if prev && !short_above => {
                // Bearish crossover: check RSI confirmation
                if (30.0..=60.0).contains(&current_rsi) {
                    info!(
                        symbol = %bar.symbol,
                        rsi = %current_rsi,
                        "EMA crossover DOWN with confirmed RSI -> SELL signal"
                    );
                    Signal::Sell
                } else {
                    debug!(
                        symbol = %bar.symbol,
                        rsi = %current_rsi,
                        "EMA crossed DOWN but RSI out of range -> HOLD"
                    );
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        };

        self.prev_short_above = Some(short_above);
        signal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(symbol: &str, close: f64) -> Bar {
        Bar {
            symbol: symbol.into(),
            open: close,
            high: close,
            low: close,
            close,
            volume: 1000,
            timestamp: "2024-01-01T09:30:00Z".into(),
        }
    }

    #[test]
    fn holds_when_insufficient_data() {
        let mut strat = EmaRsiStrategy::new("AAPL", 5, 10, 14);
        for _ in 0..14 {
            assert_eq!(strat.on_bar(&bar("AAPL", 100.0)), Signal::Hold);
        }
    }
}
