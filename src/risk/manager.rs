//! Risk manager implementation.

use crate::market::types::Position;
use crate::strategy::Signal;
use thiserror::Error;
use tracing::{info, warn};

/// Reasons why the risk manager may reject a signal.
#[derive(Debug, Error, PartialEq)]
pub enum RiskReject {
    #[error("Daily loss limit reached (realized P&L: {realized_pl:.2})")]
    DailyLossLimitReached { realized_pl: f64 },

    #[error("Maximum position size exceeded (requested: {requested}, limit: {limit})")]
    PositionSizeExceeded { requested: f64, limit: f64 },

    #[error("Maximum open positions reached ({count}/{limit})")]
    MaxOpenPositions { count: usize, limit: usize },

    #[error("Trading blocked by account or system state")]
    TradingBlocked,

    #[error("Duplicate order: already have an active {side} position in {symbol}")]
    DuplicateOrder { symbol: String, side: String },
}

/// Risk parameters that constrain strategy signals.
#[derive(Debug, Clone)]
pub struct RiskConfig {
    /// Maximum number of shares/contracts per order.
    pub max_position_size: f64,
    /// Maximum number of simultaneously open positions.
    pub max_open_positions: usize,
    /// Maximum portfolio loss per day (negative value, e.g. -500.0 means $500 loss).
    pub daily_loss_limit: f64,
    /// Whether trading is globally enabled.
    pub trading_enabled: bool,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_position_size: 10.0,
            max_open_positions: 5,
            daily_loss_limit: -500.0,
            trading_enabled: true,
        }
    }
}

/// Approved order produced by the risk manager.
#[derive(Debug, Clone)]
pub struct ApprovedOrder {
    pub symbol: String,
    pub signal: Signal,
    /// Number of shares to buy/sell after position sizing.
    pub qty: f64,
}

/// The risk manager processes signals and either approves or rejects them.
pub struct RiskManager {
    pub config: RiskConfig,
    /// Running total of realized P&L for today (loaded from storage on startup).
    pub daily_realized_pl: f64,
}

impl RiskManager {
    pub fn new(config: RiskConfig) -> Self {
        Self {
            config,
            daily_realized_pl: 0.0,
        }
    }

    /// Evaluate a strategy signal and return an approved order or a rejection reason.
    pub fn evaluate(
        &self,
        symbol: &str,
        signal: Signal,
        open_positions: &[Position],
    ) -> Result<ApprovedOrder, RiskReject> {
        if signal == Signal::Hold {
            // Hold is never sent to execution; return a dummy approval to let callers
            // handle it cleanly, but callers should check the signal before submitting.
            return Ok(ApprovedOrder {
                symbol: symbol.into(),
                signal,
                qty: 0.0,
            });
        }

        // 1. Trading globally enabled?
        if !self.config.trading_enabled {
            warn!("Risk REJECT: trading is disabled");
            return Err(RiskReject::TradingBlocked);
        }

        // 2. Daily loss limit
        if self.daily_realized_pl <= self.config.daily_loss_limit {
            warn!(
                "Risk REJECT: daily loss limit hit ({:.2})",
                self.daily_realized_pl
            );
            return Err(RiskReject::DailyLossLimitReached {
                realized_pl: self.daily_realized_pl,
            });
        }

        // 3. Open position count
        if open_positions.len() >= self.config.max_open_positions {
            warn!(
                "Risk REJECT: max open positions ({}/{})",
                open_positions.len(),
                self.config.max_open_positions
            );
            return Err(RiskReject::MaxOpenPositions {
                count: open_positions.len(),
                limit: self.config.max_open_positions,
            });
        }

        // 4. Duplicate order protection
        let already_long = open_positions.iter().any(|p| p.symbol == symbol);
        if already_long && signal == Signal::Buy {
            warn!("Risk REJECT: duplicate BUY for {}", symbol);
            return Err(RiskReject::DuplicateOrder {
                symbol: symbol.into(),
                side: "buy".into(),
            });
        }

        // 5. Position size (simple fixed sizing for now)
        let qty = self.config.max_position_size;

        info!(
            "Risk APPROVE: {} {:?} qty={}",
            symbol, signal, qty
        );

        Ok(ApprovedOrder {
            symbol: symbol.into(),
            signal,
            qty,
        })
    }

    /// Record realized P&L from a fill (call after each fill).
    pub fn record_realized_pl(&mut self, amount: f64) {
        self.daily_realized_pl += amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::types::{Position, Side};

    fn make_position(symbol: &str) -> Position {
        Position {
            symbol: symbol.into(),
            side: Side::Long,
            qty: 5.0,
            avg_entry_price: 100.0,
            market_value: 500.0,
            unrealized_pl: 0.0,
        }
    }

    #[test]
    fn hold_always_approved() {
        let rm = RiskManager::new(RiskConfig::default());
        let result = rm.evaluate("AAPL", Signal::Hold, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().qty, 0.0);
    }

    #[test]
    fn buy_approved_when_no_positions() {
        let rm = RiskManager::new(RiskConfig::default());
        let result = rm.evaluate("AAPL", Signal::Buy, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().qty, 10.0);
    }

    #[test]
    fn trading_blocked_when_disabled() {
        let config = RiskConfig {
            trading_enabled: false,
            ..Default::default()
        };
        let rm = RiskManager::new(config);
        let result = rm.evaluate("AAPL", Signal::Buy, &[]);
        assert_eq!(result.unwrap_err(), RiskReject::TradingBlocked);
    }

    #[test]
    fn daily_loss_limit_rejects() {
        let mut rm = RiskManager::new(RiskConfig::default());
        rm.daily_realized_pl = -600.0; // Exceeds -500 limit
        let result = rm.evaluate("AAPL", Signal::Buy, &[]);
        assert!(matches!(
            result.unwrap_err(),
            RiskReject::DailyLossLimitReached { .. }
        ));
    }

    #[test]
    fn max_open_positions_rejects() {
        let config = RiskConfig {
            max_open_positions: 2,
            ..Default::default()
        };
        let rm = RiskManager::new(config);
        let positions = vec![
            make_position("AAPL"),
            make_position("MSFT"),
        ];
        let result = rm.evaluate("GOOG", Signal::Buy, &positions);
        assert!(matches!(
            result.unwrap_err(),
            RiskReject::MaxOpenPositions { .. }
        ));
    }

    #[test]
    fn duplicate_buy_rejects() {
        let rm = RiskManager::new(RiskConfig::default());
        let positions = vec![make_position("AAPL")];
        let result = rm.evaluate("AAPL", Signal::Buy, &positions);
        assert!(matches!(
            result.unwrap_err(),
            RiskReject::DuplicateOrder { .. }
        ));
    }
}
