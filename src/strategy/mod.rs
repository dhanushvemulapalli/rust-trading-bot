//! Strategy module — strategy trait and concrete implementations.
//!
//! A strategy must not know about Alpaca, order submission, or risk management.
//! It only converts market data into signals.

pub mod ema_crossover;
pub mod ema_rsi;

use crate::market::types::Bar;

/// The output of a strategy after processing a bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Buy,
    Sell,
    /// No actionable view — stay flat or hold current position.
    Hold,
}

/// All strategies implement this trait.
///
/// Strategies are stateful (they maintain price history, indicator state, etc.)
/// and process one bar at a time.
pub trait Strategy: Send {
    /// Called once for each new bar.
    ///
    /// Returns a `Signal` indicating the strategy's current view.
    fn on_bar(&mut self, bar: &Bar) -> Signal;

    /// Human-readable name of the strategy (for logging).
    fn name(&self) -> &str;
}
