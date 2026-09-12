//! EMA Crossover strategy.
//!
//! Generates a BUY signal when the short EMA crosses above the long EMA,
//! and a SELL signal when it crosses below.
//!
//! This strategy is primarily a proof-of-concept for validating the pipeline.
//! Do not assume it is profitable.

use crate::market::{indicators::ema, types::Bar};
use crate::strategy::{Signal, Strategy};
use tracing::{debug, info};

/// EMA crossover strategy (configurable short/long periods).
///
/// # Example
/// ```
/// use trading_bot::strategy::ema_crossover::EmaCrossover;
/// let strategy = EmaCrossover::new("AAPL", 20, 50);
/// ```
pub struct EmaCrossover {
    symbol: String,
    short_period: usize,
    long_period: usize,
    prices: Vec<f64>,
    /// Previous signal state (to detect crossovers, not just level)
    prev_short_above: Option<bool>,
}

impl EmaCrossover {
    /// Create a new EMA crossover strategy.
    ///
    /// # Panics
    /// Panics if `short_period >= long_period` or if either period is 0.
    pub fn new(symbol: impl Into<String>, short_period: usize, long_period: usize) -> Self {
        assert!(short_period > 0, "short_period must be > 0");
        assert!(long_period > short_period, "long_period must be > short_period");
        Self {
            symbol: symbol.into(),
            short_period,
            long_period,
            prices: Vec::new(),
            prev_short_above: None,
        }
    }
}

impl Strategy for EmaCrossover {
    fn name(&self) -> &str {
        "EmaCrossover"
    }

    fn on_bar(&mut self, bar: &Bar) -> Signal {
        if bar.symbol != self.symbol {
            return Signal::Hold;
        }

        self.prices.push(bar.close);

        // Keep history bounded to avoid unbounded memory growth.
        // Keep 3× the long period for EMA stability.
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

        let short_above = short_ema > long_ema;

        debug!(
            symbol = %bar.symbol,
            short_ema = %format!("{:.4}", short_ema),
            long_ema  = %format!("{:.4}", long_ema),
            short_above = %short_above,
            "EMA values"
        );

        let signal = match self.prev_short_above {
            None => Signal::Hold,
            Some(prev) if !prev && short_above => {
                info!(
                    symbol = %bar.symbol,
                    "EMA{} crossed ABOVE EMA{} → BUY signal",
                    self.short_period, self.long_period
                );
                Signal::Buy
            }
            Some(prev) if prev && !short_above => {
                info!(
                    symbol = %bar.symbol,
                    "EMA{} crossed BELOW EMA{} → SELL signal",
                    self.short_period, self.long_period
                );
                Signal::Sell
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
    use crate::market::types::Bar;

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
    fn hold_until_enough_data() {
        let mut strat = EmaCrossover::new("AAPL", 5, 10);
        for _ in 0..9 {
            assert_eq!(strat.on_bar(&bar("AAPL", 100.0)), Signal::Hold);
        }
    }

    #[test]
    fn ignores_other_symbols() {
        let mut strat = EmaCrossover::new("AAPL", 5, 10);
        // Enough data to potentially signal, but wrong symbol
        for i in 0..20 {
            let signal = strat.on_bar(&bar("MSFT", i as f64 * 10.0));
            assert_eq!(signal, Signal::Hold, "Should ignore MSFT bars");
        }
    }

    #[test]
    fn buy_signal_on_crossover() {
        let mut strat = EmaCrossover::new("AAPL", 5, 10);
        // Feed flat prices then an upward jump
        let flat = vec![100.0_f64; 20];
        for p in &flat {
            strat.on_bar(&bar("AAPL", *p));
        }
        // Now push prices sharply upward to force short EMA above long EMA
        let signals: Vec<Signal> = (0..30)
            .map(|i| strat.on_bar(&bar("AAPL", 200.0 + i as f64 * 10.0)))
            .collect();
        assert!(
            signals.contains(&Signal::Buy),
            "Expected at least one BUY signal on crossover"
        );
    }

    #[test]
    fn sell_signal_on_crossover() {
        let mut strat = EmaCrossover::new("AAPL", 5, 10);
        // Force short above long first
        for i in 0..30 {
            strat.on_bar(&bar("AAPL", 200.0 + i as f64 * 10.0));
        }
        // Now drop prices sharply to force short below long
        let signals: Vec<Signal> = (0..30)
            .map(|i| strat.on_bar(&bar("AAPL", 500.0 - i as f64 * 20.0)))
            .collect();
        assert!(
            signals.contains(&Signal::Sell),
            "Expected at least one SELL signal on downward crossover"
        );
    }
}
