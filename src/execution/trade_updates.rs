//! Trade-updates WebSocket — receives real-time order lifecycle events from Alpaca.

use crate::config::Config;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

/// An order event received from the trading stream.
#[derive(Debug, Clone)]
pub struct OrderEvent {
    pub order_id: Uuid,
    pub event: String,
    pub symbol: Option<String>,
    pub side: Option<String>,
    pub filled_qty: f64,
    pub filled_avg_price: f64,
}

#[derive(Debug, Serialize)]
struct AuthMsg {
    action: String,
    data: AuthData,
}

#[derive(Debug, Serialize)]
struct AuthData {
    key_id: String,
    secret_key: String,
}

#[derive(Debug, Deserialize)]
struct TradeUpdateMsg {
    #[allow(dead_code)]
    stream: Option<String>,
    data: Option<TradeUpdateData>,
}

#[derive(Debug, Deserialize)]
struct TradeUpdateData {
    event: String,
    order: TradeOrder,
}

#[derive(Debug, Deserialize)]
struct TradeOrder {
    id: Uuid,
    symbol: Option<String>,
    side: Option<String>,
    filled_qty: Option<String>,
    filled_avg_price: Option<String>,
}

/// Connect to Alpaca trading stream and forward order events through `tx`.
pub async fn run_trade_updates(config: Config, tx: mpsc::Sender<OrderEvent>) -> Result<()> {
    let mut backoff_secs = 1;
    loop {
        if tx.is_closed() {
            info!("Trade update receiver dropped — shutting down");
            return Ok(());
        }

        info!("Connecting to trade-updates WebSocket: {}", config.trading_ws_url);
        let url = match url::Url::parse(&config.trading_ws_url) {
            Ok(u) => u,
            Err(e) => {
                error!("Invalid trading WS URL: {}", e);
                return Err(e.into());
            }
        };

        match connect_async(url).await {
            Ok((ws, _)) => {
                backoff_secs = 1;
                let (mut write, mut read) = ws.split();

                // Authenticate
                let auth = serde_json::to_string(&AuthMsg {
                    action: "listen".into(),
                    data: AuthData {
                        key_id: config.api_key.clone(),
                        secret_key: config.api_secret.clone(),
                    },
                })?;
                if let Err(e) = write.send(Message::Text(auth)).await {
                    warn!("Failed to send trade updates auth msg: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
                info!("Trade updates authenticated");

                // Subscribe
                let sub = r#"{"action":"listen","data":{"streams":["trade_updates"]}}"#;
                if let Err(e) = write.send(Message::Text(sub.into())).await {
                    warn!("Failed to send trade updates subscribe msg: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }

                while let Some(msg_result) = read.next().await {
                    match msg_result {
                        Ok(Message::Text(text)) => {
                            match serde_json::from_str::<TradeUpdateMsg>(&text) {
                                Ok(msg) => {
                                    if let Some(data) = msg.data {
                                        let filled_qty = data.order.filled_qty
                                            .as_deref()
                                            .unwrap_or("0")
                                            .parse::<f64>()
                                            .unwrap_or(0.0);
                                        let filled_avg_price = data.order.filled_avg_price
                                            .as_deref()
                                            .unwrap_or("0")
                                            .parse::<f64>()
                                            .unwrap_or(0.0);
                                        let event = OrderEvent {
                                            order_id: data.order.id,
                                            event: data.event.clone(),
                                            symbol: data.order.symbol,
                                            side: data.order.side,
                                            filled_qty,
                                            filled_avg_price,
                                        };
                                        info!(
                                            order_id = %event.order_id,
                                            event = %event.event,
                                            filled_qty = %filled_qty,
                                            "Trade update received"
                                        );
                                        if tx.send(event).await.is_err() {
                                            info!("Trade update receiver dropped — shutting down");
                                            return Ok(());
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse trade update: {} — raw: {}", e, text);
                                }
                            }
                        }
                        Ok(Message::Ping(p)) => {
                            let _ = write.send(Message::Pong(p)).await;
                        }
                        Ok(Message::Close(_)) => {
                            warn!("Trade updates WebSocket closed, reconnecting...");
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Trade updates WebSocket read error: {}, will reconnect...", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Trade updates WS connect failed: {}. Retrying in {}s...", e, backoff_secs);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(30);
    }
}
