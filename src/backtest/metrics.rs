//! Performance metrics calculation for backtests.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestReport {
    pub initial_capital: f64,
    pub final_capital: f64,
    pub total_pnl: f64,
    pub return_pct: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate_pct: f64,
    pub profit_factor: f64,
    pub max_drawdown_pct: f64,
}

impl std::fmt::Display for BacktestReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "─────────────────────────────────────────")?;
        writeln!(f, "           BACKTEST RESULTS              ")?;
        writeln!(f, "─────────────────────────────────────────")?;
        writeln!(f, "Initial Capital:      ${:.2}", self.initial_capital)?;
        writeln!(f, "Final Capital:        ${:.2}", self.final_capital)?;
        writeln!(f, "Total Net Profit:     ${:.2} ({:+.2}%)", self.total_pnl, self.return_pct)?;
        writeln!(f, "Total Closed Trades:  {}", self.total_trades)?;
        writeln!(f, "Winning Trades:       {}", self.winning_trades)?;
        writeln!(f, "Losing Trades:        {}", self.losing_trades)?;
        writeln!(f, "Win Rate:             {:.1}%", self.win_rate_pct)?;
        writeln!(f, "Profit Factor:        {:.2}", self.profit_factor)?;
        writeln!(f, "Max Drawdown:         {:.2}%", self.max_drawdown_pct)?;
        writeln!(f, "─────────────────────────────────────────")?;
        Ok(())
    }
}

/// Calculate Maximum Drawdown percentage from an equity curve.
pub fn calculate_max_drawdown(equity_curve: &[f64]) -> f64 {
    if equity_curve.is_empty() {
        return 0.0;
    }
    let mut peak = equity_curve[0];
    let mut max_dd = 0.0;

    for &equity in equity_curve {
        if equity > peak {
            peak = equity;
        } else if peak > 0.0 {
            let dd = (peak - equity) / peak * 100.0;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }

    max_dd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_drawdown() {
        let curve = vec![100.0, 110.0, 120.0, 90.0, 95.0, 130.0];
        // Peak is 120, trough is 90 -> dd = (120 - 90) / 120 = 25%
        let dd = calculate_max_drawdown(&curve);
        assert!((dd - 25.0).abs() < 1e-5);
    }
}
