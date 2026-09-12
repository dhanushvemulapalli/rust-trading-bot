//! Trading bot — entry point.
//!
//! Wires together:
//!   Config → AlpacaClient → MarketData → Strategy → RiskManager → OrderManager
//!
//! All trading is paper-trading by default.

mod alpaca;
mod config;
mod execution;
mod market;
mod risk;
mod storage;
mod strategy;

use crate::alpaca::client::AlpacaClient;
use crate::alpaca::market_data::run_collector;
use crate::config::Config;
use crate::execution::order_manager::{OrderManager, OrderStatus};
use crate::execution::trade_updates::run_trade_updates;
use crate::market::types::Bar;
use crate::risk::manager::{RiskConfig, RiskManager};
use crate::storage::database::Db;
use crate::strategy::{Signal, Strategy};
use crate::strategy::ema_crossover::EmaCrossover;
use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Symbols to trade (initial paper-trading symbols).
const SYMBOLS: &[&str] = &["AAPL", "MSFT"];
/// EMA periods for the crossover strategy.
const EMA_SHORT: usize = 20;
const EMA_LONG: usize = 50;
/// Channel buffer sizes.
const BAR_CHAN_SIZE: usize = 1024;
const ORDER_EVENT_CHAN_SIZE: usize = 256;

#[tokio::main]
async fn main() -> Result<()> {
    // ── Logging ─────────────────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("trading_bot=debug".parse()?)
                .add_directive("warn".parse()?),
        )
        .init();

    // ── Configuration ────────────────────────────────────────────────────────
    let config = Config::load()?;
    if !config.use_paper {
        error!("USE_PAPER is false — live trading is not enabled in this build. Exiting.");
        std::process::exit(1);
    }
    info!("Starting trading bot (paper trading)");

    // ── Database ─────────────────────────────────────────────────────────────
    let db = Db::connect(&config.database_url).await?;
    info!("Database ready");

    // ── Alpaca client ────────────────────────────────────────────────────────
    let client = AlpacaClient::new(&config)?;

    // Milestone 1: verify connectivity
    let account = client.get_account().await?;
    info!(
        account_id = %account.id,
        status = %account.status,
        cash = %account.cash,
        buying_power = %account.buying_power,
        "Alpaca account loaded"
    );

    // ── Channels ─────────────────────────────────────────────────────────────
    let (bar_tx, mut bar_rx) = mpsc::channel::<Bar>(BAR_CHAN_SIZE);
    let (order_event_tx, mut order_event_rx) = mpsc::channel(ORDER_EVENT_CHAN_SIZE);

    // ── Spawn market-data collector ──────────────────────────────────────────
    let symbols: Vec<String> = SYMBOLS.iter().map(|s| s.to_string()).collect();
    let collector_config = config.clone();
    let collector_handle = tokio::spawn(async move {
        if let Err(e) = run_collector(collector_config, symbols, bar_tx).await {
            error!("Market data collector error: {}", e);
        }
    });

    // ── Spawn trade-updates listener ─────────────────────────────────────────
    let tu_config = config.clone();
    let trade_update_handle = tokio::spawn(async move {
        if let Err(e) = run_trade_updates(tu_config, order_event_tx).await {
            error!("Trade updates error: {}", e);
        }
    });

    // ── Strategy + risk manager ──────────────────────────────────────────────
    let mut strategies: Vec<Box<dyn Strategy>> = SYMBOLS
        .iter()
        .map(|&sym| -> Box<dyn Strategy> {
            Box::new(EmaCrossover::new(sym, EMA_SHORT, EMA_LONG))
        })
        .collect();

    let risk_config = RiskConfig::default();
    let mut risk_manager = RiskManager::new(risk_config);
    let mut order_manager = OrderManager::new(client);

    // ── Main event loop ──────────────────────────────────────────────────────
    loop {
        tokio::select! {
            // Process incoming bars
            Some(bar) = bar_rx.recv() => {
                // Persist bar
                if let Err(e) = db.insert_bar(
                    &bar.symbol,
                    bar.open, bar.high, bar.low, bar.close,
                    bar.volume as i64,
                    &bar.timestamp,
                ).await {
                    error!("Failed to persist bar: {}", e);
                }

                // Run each strategy on the bar
                for strategy in &mut strategies {
                    let signal = strategy.on_bar(&bar);

                    if signal != Signal::Hold {
                        info!(
                            symbol = %bar.symbol,
                            strategy = %strategy.name(),
                            signal = ?signal,
                            "Signal generated"
                        );

                        // Persist signal
                        let signal_str = format!("{:?}", signal);
                        if let Err(e) = db.insert_signal(&bar.symbol, &signal_str, &bar.timestamp).await {
                            error!("Failed to persist signal: {}", e);
                        }
                    }

                    // Risk evaluation
                    // TODO: pass real open positions from account once position tracking is built
                    let open_positions = vec![];
                    match risk_manager.evaluate(&bar.symbol, signal, &open_positions) {
                        Ok(approved) if approved.signal != Signal::Hold => {
                            match order_manager.submit(&approved).await {
                                Ok(Some(order_id)) => {
                                    info!(order_id = %order_id, "Order submitted");
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    error!("Order submission failed: {}", e);
                                }
                            }
                        }
                        Ok(_) => {} // Hold — no action
                        Err(reject) => {
                            warn!("Risk rejected signal: {}", reject);
                        }
                    }
                }
            }

            // Process order events from trade-updates
            Some(event) = order_event_rx.recv() => {
                let status = match event.event.as_str() {
                    "new" | "accepted"     => OrderStatus::Accepted,
                    "fill"                 => OrderStatus::Filled,
                    "partial_fill"         => OrderStatus::PartiallyFilled {
                        filled_qty: event.filled_qty as u64,
                    },
                    "canceled" | "expired" => OrderStatus::Canceled,
                    "rejected"             => OrderStatus::Rejected,
                    _                      => OrderStatus::Pending,
                };
                order_manager.on_order_update(event.order_id, status, event.filled_qty);
            }

            else => {
                warn!("All channels closed — shutting down main loop");
                break;
            }
        }
    }

    // ── Shutdown ─────────────────────────────────────────────────────────────
    collector_handle.abort();
    trade_update_handle.abort();
    info!("Trading bot shut down cleanly");
    Ok(())
}
