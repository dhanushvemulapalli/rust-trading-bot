use trading_bot::alpaca::client::AlpacaClient;
use trading_bot::alpaca::market_data::run_collector;
use trading_bot::config::Config;
use trading_bot::execution::order_manager::{OrderManager, OrderStatus};
use trading_bot::execution::position_tracker::PositionTracker;
use trading_bot::execution::trade_updates::run_trade_updates;
use trading_bot::market::types::{Bar, Position};
use trading_bot::risk::manager::{RiskConfig, RiskManager};
use trading_bot::storage::database::Db;
use trading_bot::strategy::{Signal, Strategy};
use trading_bot::strategy::ema_rsi::EmaRsiStrategy;
use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Symbols to trade (initial paper-trading symbols).
const SYMBOLS: &[&str] = &["AAPL", "MSFT"];
/// Strategy parameters.
const EMA_SHORT: usize = 20;
const EMA_LONG: usize = 50;
const RSI_PERIOD: usize = 14;
/// Channel buffer sizes.
const BAR_CHAN_SIZE: usize = 1024;
const ORDER_EVENT_CHAN_SIZE: usize = 256;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("trading_bot=debug".parse()?)
                .add_directive("warn".parse()?),
        )
        .init();

    let config = Config::load()?;
    if !config.use_paper {
        error!("USE_PAPER is false - live trading is not enabled in this build. Exiting.");
        std::process::exit(1);
    }
    info!("Starting trading bot (paper trading)");

    let db = Db::connect(&config.database_url).await?;
    info!("Database ready");

    let client = AlpacaClient::new(&config)?;

    let account = client.get_account().await?;
    info!(
        account_id = %account.id,
        status = %account.status,
        cash = %account.cash,
        buying_power = %account.buying_power,
        "Alpaca account loaded"
    );

    // Load initial positions
    let mut position_tracker = PositionTracker::new();
    match client.get_positions().await {
        Ok(raw_positions) => {
            let domain_positions: Vec<Position> = raw_positions.iter().map(|p| p.to_domain()).collect();
            for pos in &domain_positions {
                if let Err(e) = db.upsert_position(pos).await {
                    warn!("Failed to persist initial position: {}", e);
                }
            }
            position_tracker.sync_positions(domain_positions);
            info!("Positions loaded from Alpaca (count: {})", position_tracker.get_positions().len());
        }
        Err(e) => {
            warn!("Failed to load initial positions from Alpaca: {}", e);
        }
    }

    let (bar_tx, mut bar_rx) = mpsc::channel::<Bar>(BAR_CHAN_SIZE);
    let (order_event_tx, mut order_event_rx) = mpsc::channel(ORDER_EVENT_CHAN_SIZE);

    let symbols: Vec<String> = SYMBOLS.iter().map(|s| s.to_string()).collect();
    let collector_config = config.clone();
    let collector_handle = tokio::spawn(async move {
        if let Err(e) = run_collector(collector_config, symbols, bar_tx).await {
            error!("Market data collector error: {}", e);
        }
    });

    let tu_config = config.clone();
    let trade_update_handle = tokio::spawn(async move {
        if let Err(e) = run_trade_updates(tu_config, order_event_tx).await {
            error!("Trade updates error: {}", e);
        }
    });

    let mut strategies: Vec<Box<dyn Strategy>> = SYMBOLS
        .iter()
        .map(|&sym| -> Box<dyn Strategy> {
            Box::new(EmaRsiStrategy::new(sym, EMA_SHORT, EMA_LONG, RSI_PERIOD))
        })
        .collect();

    let risk_config = RiskConfig::default();
    let risk_manager = RiskManager::new(risk_config);
    let mut order_manager = OrderManager::new(client);

    loop {
        tokio::select! {
            Some(bar) = bar_rx.recv() => {
                if let Err(e) = db.insert_bar(
                    &bar.symbol,
                    bar.open, bar.high, bar.low, bar.close,
                    bar.volume as i64,
                    &bar.timestamp,
                ).await {
                    error!("Failed to persist bar: {}", e);
                }

                for strategy in &mut strategies {
                    let signal = strategy.on_bar(&bar);

                    if signal != Signal::Hold {
                        info!(
                            symbol = %bar.symbol,
                            strategy = %strategy.name(),
                            signal = ?signal,
                            "Signal generated"
                        );

                        let signal_str = format!("{:?}", signal);
                        if let Err(e) = db.insert_signal(&bar.symbol, &signal_str, &bar.timestamp).await {
                            error!("Failed to persist signal: {}", e);
                        }
                    }

                    // Feed active positions and current net worth (70% sizing rule)
                    let open_positions = position_tracker.get_positions();
                    let net_worth = account.equity.parse::<f64>().unwrap_or(10_000.0);
                    match risk_manager.evaluate(&bar.symbol, signal, &open_positions, net_worth, bar.close) {
                        Ok(approved) if approved.signal != Signal::Hold => {
                            match order_manager.submit(&approved).await {
                                Ok(Some(order_id)) => {
                                    info!(order_id = %order_id, "Order submitted");
                                    let side_str = format!("{:?}", approved.signal);
                                    if let Err(e) = db.upsert_order(
                                        order_id,
                                        &format!("tbot-{}", order_id),
                                        &approved.symbol,
                                        &side_str,
                                        approved.qty,
                                        0.0,
                                        "submitted",
                                    ).await {
                                        error!("Failed to persist submitted order: {}", e);
                                    }
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    error!("Order submission failed: {}", e);
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(reject) => {
                            warn!("Risk rejected signal: {}", reject);
                        }
                    }
                }
            }

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
                order_manager.on_order_update(event.order_id, status.clone(), event.filled_qty);

                let status_str = format!("{:?}", status);
                if let Err(e) = db.update_order_status(event.order_id, &status_str, event.filled_qty).await {
                    error!("Failed to update order status in DB: {}", e);
                }

                // Update position tracker on fills
                if event.event == "fill" || event.event == "partial_fill" {
                    if let (Some(sym), Some(side)) = (event.symbol.as_deref(), event.side.as_deref()) {
                        let signal = if side.eq_ignore_ascii_case("buy") {
                            Signal::Buy
                        } else {
                            Signal::Sell
                        };
                        position_tracker.on_fill(sym, signal, event.filled_qty, event.filled_avg_price);

                        if let Some(pos) = position_tracker.get_position(sym) {
                            let _ = db.upsert_position(pos).await;
                        } else {
                            let _ = db.delete_position(sym).await;
                        }
                    }
                }
            }

            else => {
                warn!("All channels closed - shutting down main loop");
                break;
            }
        }
    }

    collector_handle.abort();
    trade_update_handle.abort();
    info!("Trading bot shut down cleanly");
    Ok(())
}
