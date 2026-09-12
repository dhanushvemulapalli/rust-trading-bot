//! Position tracker — keeps real-time portfolio positions synchronized with broker and fills.

use crate::market::types::{Position, Side};
use crate::strategy::Signal;
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Default)]
pub struct PositionTracker {
    positions: HashMap<String, Position>,
}

impl PositionTracker {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }

    /// Overwrite positions from Alpaca REST sync (e.g. on startup or reconciliation).
    pub fn sync_positions(&mut self, positions: Vec<Position>) {
        self.positions.clear();
        for pos in positions {
            info!(
                symbol = %pos.symbol,
                side = ?pos.side,
                qty = %pos.qty,
                avg_price = %pos.avg_entry_price,
                "Position synced from broker"
            );
            self.positions.insert(pos.symbol.clone(), pos);
        }
    }

    /// Return a snapshot of all active positions.
    pub fn get_positions(&self) -> Vec<Position> {
        self.positions.values().cloned().collect()
    }

    /// Get position for a specific symbol.
    pub fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    /// Apply an order fill to update the position.
    pub fn on_fill(&mut self, symbol: &str, signal: Signal, fill_qty: f64, fill_price: f64) {
        match signal {
            Signal::Buy => {
                if let Some(pos) = self.positions.get_mut(symbol) {
                    if pos.side == Side::Long {
                        let total_cost = (pos.qty * pos.avg_entry_price) + (fill_qty * fill_price);
                        pos.qty += fill_qty;
                        if pos.qty > 0.0 {
                            pos.avg_entry_price = total_cost / pos.qty;
                        }
                    } else {
                        // Covering a short
                        if fill_qty >= pos.qty {
                            let rem = fill_qty - pos.qty;
                            if rem > 0.0 {
                                *pos = Position {
                                    symbol: symbol.into(),
                                    side: Side::Long,
                                    qty: rem,
                                    avg_entry_price: fill_price,
                                    market_value: rem * fill_price,
                                    unrealized_pl: 0.0,
                                };
                            } else {
                                self.positions.remove(symbol);
                            }
                        } else {
                            pos.qty -= fill_qty;
                        }
                    }
                } else {
                    self.positions.insert(
                        symbol.into(),
                        Position {
                            symbol: symbol.into(),
                            side: Side::Long,
                            qty: fill_qty,
                            avg_entry_price: fill_price,
                            market_value: fill_qty * fill_price,
                            unrealized_pl: 0.0,
                        },
                    );
                }
            }
            Signal::Sell => {
                if let Some(pos) = self.positions.get_mut(symbol) {
                    if pos.side == Side::Long {
                        if fill_qty >= pos.qty {
                            self.positions.remove(symbol);
                        } else {
                            pos.qty -= fill_qty;
                        }
                    } else {
                        // Adding to short
                        let total_cost = (pos.qty * pos.avg_entry_price) + (fill_qty * fill_price);
                        pos.qty += fill_qty;
                        if pos.qty > 0.0 {
                            pos.avg_entry_price = total_cost / pos.qty;
                        }
                    }
                } else {
                    self.positions.insert(
                        symbol.into(),
                        Position {
                            symbol: symbol.into(),
                            side: Side::Short,
                            qty: fill_qty,
                            avg_entry_price: fill_price,
                            market_value: fill_qty * fill_price,
                            unrealized_pl: 0.0,
                        },
                    );
                }
            }
            Signal::Hold => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_opens_and_closes_long() {
        let mut tracker = PositionTracker::new();
        assert!(tracker.get_positions().is_empty());

        // Buy 10 @ 100
        tracker.on_fill("AAPL", Signal::Buy, 10.0, 100.0);
        let pos = tracker.get_position("AAPL").unwrap();
        assert_eq!(pos.qty, 10.0);
        assert_eq!(pos.avg_entry_price, 100.0);
        assert_eq!(pos.side, Side::Long);

        // Buy another 10 @ 110 -> avg 105
        tracker.on_fill("AAPL", Signal::Buy, 10.0, 110.0);
        let pos = tracker.get_position("AAPL").unwrap();
        assert_eq!(pos.qty, 20.0);
        assert_eq!(pos.avg_entry_price, 105.0);

        // Sell 20 -> closed
        tracker.on_fill("AAPL", Signal::Sell, 20.0, 120.0);
        assert!(tracker.get_position("AAPL").is_none());
    }
}
