//! Example runner: Backtesting EMA vs EMA + RSI Momentum Filter on historical/synthetic data.
//!
//! Run with:
//! cargo run --example run_backtest

use trading_bot::backtest::data::generate_synthetic_bars;
use trading_bot::backtest::engine::{BacktestConfig, BacktestEngine};
use trading_bot::strategy::ema_crossover::EmaCrossover;
use trading_bot::strategy::ema_rsi::EmaRsiStrategy;

fn main() {
    println!("Generating 600 bars of synthetic market data for AAPL...");
    let bars = generate_synthetic_bars("AAPL", 150.0, 600);

    let config = BacktestConfig {
        initial_capital: 10_000.0,
        slippage_pct: 0.0005, // 0.05% slippage
        commission: 1.0,      // $1 per trade
        position_size: 20.0,  // 20 shares per order
    };

    println!("\n[1] Running Pure EMA(10, 30) Crossover...");
    let mut ema_strat = EmaCrossover::new("AAPL", 10, 30);
    let mut engine_ema = BacktestEngine::new(config.clone());
    let report_ema = engine_ema.run(&mut ema_strat, &bars);
    println!("{}", report_ema);

    println!("\n[2] Running EMA(10, 30) + RSI(14) Filtered Strategy...");
    let mut ema_rsi_strat = EmaRsiStrategy::new("AAPL", 10, 30, 14);
    let mut engine_rsi = BacktestEngine::new(config);
    let report_rsi = engine_rsi.run(&mut ema_rsi_strat, &bars);
    println!("{}", report_rsi);
}
