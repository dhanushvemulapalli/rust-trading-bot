//! Event-driven historical backtesting simulation engine.

use crate::backtest::metrics::{calculate_max_drawdown, BacktestReport};
use crate::market::types::Bar;
use crate::strategy::{Signal, Strategy};

#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub initial_capital: f64,
    /// Slippage fraction (e.g. 0.0005 for 0.05% slippage per fill)
    pub slippage_pct: f64,
    /// Fixed commission per order in dollars
    pub commission: f64,
    /// Fixed position size in shares to trade
    pub position_size: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10_000.0,
            slippage_pct: 0.0005,
            commission: 0.0,
            position_size: 10.0,
        }
    }
}

pub struct BacktestEngine {
    config: BacktestConfig,
    cash: f64,
    position_qty: f64,
    entry_price: f64,
    equity_curve: Vec<f64>,
    closed_trades: Vec<f64>, // List of realized P&L per trade
}

impl BacktestEngine {
    pub fn new(config: BacktestConfig) -> Self {
        let initial = config.initial_capital;
        Self {
            config,
            cash: initial,
            position_qty: 0.0,
            entry_price: 0.0,
            equity_curve: vec![initial],
            closed_trades: Vec::new(),
        }
    }

    /// Run the strategy across historical bars and return a performance report.
    pub fn run(&mut self, strategy: &mut dyn Strategy, bars: &[Bar]) -> BacktestReport {
        for bar in bars {
            let signal = strategy.on_bar(bar);
            match signal {
                Signal::Buy => {
                    // Only open if currently flat
                    if self.position_qty == 0.0 {
                        let fill_price = bar.close * (1.0 + self.config.slippage_pct);
                        let cost = (self.config.position_size * fill_price) + self.config.commission;
                        if self.cash >= cost {
                            self.cash -= cost;
                            self.position_qty = self.config.position_size;
                            self.entry_price = fill_price;
                        }
                    }
                }
                Signal::Sell => {
                    // Close existing long position
                    if self.position_qty > 0.0 {
                        let fill_price = bar.close * (1.0 - self.config.slippage_pct);
                        let proceeds = (self.position_qty * fill_price) - self.config.commission;
                        let cost_basis = self.position_qty * self.entry_price;
                        let pnl = proceeds - cost_basis;

                        self.cash += proceeds;
                        self.closed_trades.push(pnl);
                        self.position_qty = 0.0;
                        self.entry_price = 0.0;
                    }
                }
                Signal::Hold => {}
            }

            // Current mark-to-market equity
            let current_equity = self.cash + (self.position_qty * bar.close);
            self.equity_curve.push(current_equity);
        }

        // Close any open position at the final bar price to settle P&L
        if let Some(last_bar) = bars.last() {
            if self.position_qty > 0.0 {
                let fill_price = last_bar.close * (1.0 - self.config.slippage_pct);
                let proceeds = (self.position_qty * fill_price) - self.config.commission;
                let cost_basis = self.position_qty * self.entry_price;
                let pnl = proceeds - cost_basis;

                self.cash += proceeds;
                self.closed_trades.push(pnl);
                self.position_qty = 0.0;
            }
        }

        let final_capital = self.cash;
        let total_pnl = final_capital - self.config.initial_capital;
        let return_pct = (total_pnl / self.config.initial_capital) * 100.0;

        let total_trades = self.closed_trades.len();
        let winning_trades = self.closed_trades.iter().filter(|&&p| p > 0.0).count();
        let losing_trades = self.closed_trades.iter().filter(|&&p| p < 0.0).count();
        let win_rate_pct = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let gross_profits: f64 = self.closed_trades.iter().filter(|&&p| p > 0.0).sum();
        let gross_losses: f64 = self.closed_trades.iter().filter(|&&p| p < 0.0).map(|p| p.abs()).sum();
        let profit_factor = if gross_losses > 0.0 {
            gross_profits / gross_losses
        } else if gross_profits > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        let max_drawdown_pct = calculate_max_drawdown(&self.equity_curve);

        BacktestReport {
            initial_capital: self.config.initial_capital,
            final_capital,
            total_pnl,
            return_pct,
            total_trades,
            winning_trades,
            losing_trades,
            win_rate_pct,
            profit_factor,
            max_drawdown_pct,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::data::generate_synthetic_bars;
    use crate::strategy::ema_crossover::EmaCrossover;

    #[test]
    fn backtest_runs_on_synthetic_data() {
        let bars = generate_synthetic_bars("AAPL", 100.0, 300);
        let mut strategy = EmaCrossover::new("AAPL", 10, 30);
        let mut engine = BacktestEngine::new(BacktestConfig::default());
        let report = engine.run(&mut strategy, &bars);

        assert!(report.total_trades > 0);
        assert!(report.final_capital > 0.0);
    }
}
