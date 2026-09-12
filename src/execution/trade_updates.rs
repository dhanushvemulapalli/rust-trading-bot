//! Trade-updates WebSocket — receives real-time order lifecycle events from Alpaca.

use crate::config::Config;
use anyhow::{bail, Result};
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
    pub filled_qty: f64,
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
    filled_qty: Option<String>,
}

/// Connect to Alpaca trading stream and forward order events through `tx`.
pub async fn run_trade_updates(config: Config, tx: mpsc::Sender<OrderEvent>) -> Result<()> {
    info!("Connecting to trade-updates WebSocket: {}", config.trading_ws_url);
    let url = url::Url::parse(&config.trading_ws_url)?;
    let (ws, _) = connect_async(url).await?;
    let (mut write, mut read) = ws.split();

    // Authenticate
    let auth = serde_json::to_string(&AuthMsg {
        action: "listen".into(),
        data: AuthData {
            key_id: config.api_key.clone(),
            secret_key: config.api_secret.clone(),
        },
    })?;
    write.send(Message::Text(auth)).await?;
    info!("Trade updates authenticated");

    // Subscribe
    let sub = r#"{"action":"listen","data":{"streams":["trade_updates"]}}"#;
    write.send(Message::Text(sub.into())).await?;

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
                            let event = OrderEvent {
                                order_id: data.order.id,
                                event: data.event.clone(),
                                filled_qty,
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
                write.send(Message::Pong(p)).await?;
            }
            Ok(Message::Close(_)) => {
                info!("Trade updates WebSocket closed");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                error!("Trade updates WebSocket error: {}", e);
                bail!("Trade updates WebSocket error: {}", e);
            }
        }
    }
    Ok(())
}
