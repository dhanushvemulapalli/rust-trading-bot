//! Risk manager implementation — dynamic 70% Net Worth position sizing and risk controls.

use crate::market::types::Position;
use crate::market::universe::get_instrument;
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

    #[error("Maximum portfolio exposure limit exceeded ({current_exposure:.2}/{max_exposure:.2})")]
    MaxExposureExceeded { current_exposure: f64, max_exposure: f64 },

    #[error("No open position to sell for {symbol}")]
    NoPositionToSell { symbol: String },
}

/// Risk parameters that constrain strategy signals.
#[derive(Debug, Clone)]
pub struct RiskConfig {
    /// Target percentage of total net worth to deploy (default: 0.70 for 70%)
    pub target_net_worth_pct: f64,
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
            target_net_worth_pct: 0.70,
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
    /// Number of shares/contracts/units to buy/sell after dynamic position sizing.
    pub qty: f64,
}

/// The risk manager processes signals and applies portfolio safeguards and dynamic sizing.
pub struct RiskManager {
    pub config: RiskConfig,
    /// Running total of realized P&L for today.
    pub daily_realized_pl: f64,
}

impl RiskManager {
    pub fn new(config: RiskConfig) -> Self {
        Self {
            config,
            daily_realized_pl: 0.0,
        }
    }

    /// Evaluate a strategy signal with dynamic 70% net worth capital allocation.
    pub fn evaluate(
        &self,
        symbol: &str,
        signal: Signal,
        open_positions: &[Position],
        net_worth: f64,
        market_price: f64,
    ) -> Result<ApprovedOrder, RiskReject> {
        if signal == Signal::Hold {
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

        // Handle Sell / Close signal
        if signal == Signal::Sell {
            if let Some(pos) = open_positions.iter().find(|p| p.symbol == symbol) {
                return Ok(ApprovedOrder {
                    symbol: symbol.into(),
                    signal: Signal::Sell,
                    qty: pos.qty,
                });
            } else {
                return Err(RiskReject::NoPositionToSell {
                    symbol: symbol.into(),
                });
            }
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

        // 4. Duplicate order protection (reject duplicate Buy if already long)
        let already_long = open_positions.iter().any(|p| p.symbol == symbol);
        if already_long {
            warn!("Risk REJECT: duplicate BUY for {}", symbol);
            return Err(RiskReject::DuplicateOrder {
                symbol: symbol.into(),
                side: "buy".into(),
            });
        }

        // 5. Dynamic 70% Net Worth Capital Allocation
        let target_max_exposure = net_worth * self.config.target_net_worth_pct;
        let current_exposure: f64 = open_positions
            .iter()
            .map(|p| p.market_value.abs())
            .sum();

        if current_exposure >= target_max_exposure {
            warn!(
                "Risk REJECT: max 70% exposure reached ({:.2}/{:.2})",
                current_exposure, target_max_exposure
            );
            return Err(RiskReject::MaxExposureExceeded {
                current_exposure,
                max_exposure: target_max_exposure,
            });
        }

        let available_capital = target_max_exposure - current_exposure;
        let per_trade_capital = (target_max_exposure / self.config.max_open_positions as f64).min(available_capital);

        if market_price <= 0.0 {
            return Err(RiskReject::PositionSizeExceeded {
                requested: 0.0,
                limit: per_trade_capital,
            });
        }

        let raw_qty = per_trade_capital / market_price;

        // Determine lot precision from universe instrument
        let is_fractional = get_instrument(symbol)
            .map(|i| i.is_fractional)
            .unwrap_or(false);

        let qty = if is_fractional {
            // Crypto: 4 decimal places precision
            (raw_qty * 10000.0).floor() / 10000.0
        } else {
            // Equities & Contracts: integer lots
            raw_qty.floor()
        };

        if qty <= 0.0 {
            warn!("Risk REJECT: calculated order qty is 0");
            return Err(RiskReject::PositionSizeExceeded {
                requested: 0.0,
                limit: per_trade_capital,
            });
        }

        info!(
            "Risk APPROVE: {} {:?} qty={} (70% net worth allocation: ${:.2})",
            symbol, signal, qty, per_trade_capital
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

    fn make_position(symbol: &str, market_value: f64) -> Position {
        Position {
            symbol: symbol.into(),
            side: Side::Long,
            qty: 10.0,
            avg_entry_price: market_value / 10.0,
            market_value,
            unrealized_pl: 0.0,
        }
    }

    #[test]
    fn hold_always_approved() {
        let rm = RiskManager::new(RiskConfig::default());
        let result = rm.evaluate("AAPL", Signal::Hold, &[], 100_000.0, 150.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().qty, 0.0);
    }

    #[test]
    fn dynamic_70_percent_sizing_allocates_correctly() {
        let rm = RiskManager::new(RiskConfig::default()); // 70%, 5 max positions
        // Net worth 100,000 -> 70% is 70,000. Each of 5 positions gets 14,000.
        // Price is 100 -> 14,000 / 100 = 140 shares.
        let result = rm.evaluate("AAPL", Signal::Buy, &[], 100_000.0, 100.0).unwrap();
        assert_eq!(result.qty, 140.0);
    }

    #[test]
    fn crypto_fractional_lot_precision() {
        let rm = RiskManager::new(RiskConfig::default());
        // Net worth 100,000 -> per trade 14,000. Price of BTC = 60,000 -> 14,000 / 60,000 = 0.2333 BTC
        let result = rm.evaluate("BTC/USD", Signal::Buy, &[], 100_000.0, 60_000.0).unwrap();
        assert_eq!(result.qty, 0.2333);
    }

    #[test]
    fn max_70_percent_exposure_rejection() {
        let rm = RiskManager::new(RiskConfig::default());
        // 100,000 net worth, 70,000 already deployed -> should reject new Buy
        let positions = vec![
            make_position("AAPL", 35_000.0),
            make_position("MSFT", 35_000.0),
        ];
        let result = rm.evaluate("GOOG", Signal::Buy, &positions, 100_000.0, 150.0);
        assert!(matches!(result.unwrap_err(), RiskReject::MaxExposureExceeded { .. }));
    }

    #[test]
    fn duplicate_buy_rejects() {
        let rm = RiskManager::new(RiskConfig::default());
        let positions = vec![make_position("AAPL", 5_000.0)];
        let result = rm.evaluate("AAPL", Signal::Buy, &positions, 100_000.0, 150.0);
        assert!(matches!(result.unwrap_err(), RiskReject::DuplicateOrder { .. }));
    }
}
