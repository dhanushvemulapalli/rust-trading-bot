//! Integration tests for the full bot pipeline using mocked/simulated data.
//! These tests do not connect to Alpaca.

use trading_bot::market::indicators::{ema, sma};
use trading_bot::market::types::Bar;
use trading_bot::risk::manager::{RiskConfig, RiskManager};
use trading_bot::strategy::{Signal, Strategy};
use trading_bot::strategy::ema_crossover::EmaCrossover;

fn make_bar(symbol: &str, close: f64) -> Bar {
    Bar {
        symbol: symbol.into(),
        open: close,
        high: close,
        low: close,
        close,
        volume: 1000,
        timestamp: "2024-01-15T09:30:00Z".into(),
    }
}

// ── Indicator tests ───────────────────────────────────────────────────────────

#[test]
fn ema_constant_series() {
    let prices = vec![42.0_f64; 50];
    let result = ema(&prices, 20).unwrap();
    assert!((result - 42.0).abs() < 1e-5);
}

#[test]
fn sma_basic() {
    let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];
    assert!((sma(&prices, 3).unwrap() - 40.0).abs() < 1e-5);
}

// ── Strategy tests ────────────────────────────────────────────────────────────

#[test]
fn strategy_produces_buy_on_uptrend() {
    let mut strat = EmaCrossover::new("AAPL", 5, 10);
    // Prime with flat prices
    for _ in 0..20 {
        strat.on_bar(&make_bar("AAPL", 100.0));
    }
    // Inject rising prices to trigger crossover
    let signals: Vec<Signal> = (0..40)
        .map(|i| strat.on_bar(&make_bar("AAPL", 200.0 + i as f64 * 5.0)))
        .collect();
    assert!(signals.contains(&Signal::Buy));
}

#[test]
fn strategy_hold_before_enough_data() {
    let mut strat = EmaCrossover::new("AAPL", 5, 10);
    for i in 0..9 {
        assert_eq!(strat.on_bar(&make_bar("AAPL", i as f64 * 10.0)), Signal::Hold);
    }
}

// ── Risk manager tests ────────────────────────────────────────────────────────

#[test]
fn risk_manager_approves_valid_buy() {
    let rm = RiskManager::new(RiskConfig::default());
    let result = rm.evaluate("AAPL", Signal::Buy, &[]);
    assert!(result.is_ok());
}

#[test]
fn risk_manager_rejects_when_disabled() {
    let config = RiskConfig {
        trading_enabled: false,
        ..Default::default()
    };
    let rm = RiskManager::new(config);
    assert!(rm.evaluate("AAPL", Signal::Buy, &[]).is_err());
}

#[test]
fn risk_manager_rejects_on_loss_limit() {
    let mut rm = RiskManager::new(RiskConfig::default());
    rm.daily_realized_pl = -1000.0;
    assert!(rm.evaluate("AAPL", Signal::Buy, &[]).is_err());
}
