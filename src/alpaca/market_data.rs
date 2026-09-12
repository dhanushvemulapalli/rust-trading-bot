//! Alpaca market-data WebSocket collector.
//!
//! Connects to the Alpaca market-data stream, authenticates,
//! subscribes to bars for a set of symbols, and forwards
//! received bars through an async channel.

use crate::config::Config;
use crate::market::types::Bar;
use anyhow::{bail, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

// ── Alpaca WebSocket message types ────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct AuthMsg {
    action: &'static str,
    key: String,
    secret: String,
}

#[derive(Debug, Serialize)]
struct SubscribeMsg {
    action: &'static str,
    bars: Vec<String>,
}

/// Raw bar as received from the Alpaca WebSocket stream.
#[derive(Debug, Deserialize)]
pub struct WsBar {
    #[serde(rename = "T")]
    pub msg_type: String,
    #[serde(rename = "S")]
    pub symbol: String,
    #[serde(rename = "o")]
    pub open: f64,
    #[serde(rename = "h")]
    pub high: f64,
    #[serde(rename = "l")]
    pub low: f64,
    #[serde(rename = "c")]
    pub close: f64,
    #[serde(rename = "v")]
    pub volume: u64,
    #[serde(rename = "t")]
    pub timestamp: String,
}

/// Generic wrapper for incoming WS messages.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum WsMsg {
    Array(Vec<WsEvent>),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "T")]
enum WsEvent {
    #[serde(rename = "success")]
    Success { msg: String },
    #[serde(rename = "error")]
    Error { code: u32, msg: String },
    #[serde(rename = "subscription")]
    Subscription,
    #[serde(rename = "b")]
    Bar(WsBar),
    #[serde(other)]
    Unknown,
}

/// Spawn the market-data WebSocket collector.
///
/// Sends deserialized `Bar`s through `tx`. The caller owns `rx`.
pub async fn run_collector(
    config: Config,
    symbols: Vec<String>,
    tx: mpsc::Sender<Bar>,
) -> Result<()> {
    let mut backoff_secs = 1;
    loop {
        if tx.is_closed() {
            info!("Bar receiver dropped — shutting down collector");
            return Ok(());
        }

        info!("Connecting to market-data WebSocket: {}", config.data_ws_url);
        let url = match url::Url::parse(&config.data_ws_url) {
            Ok(u) => u,
            Err(e) => {
                error!("Invalid market data WS URL: {}", e);
                return Err(e.into());
            }
        };

        match connect_async(url).await {
            Ok((ws, _)) => {
                backoff_secs = 1;
                let (mut write, mut read) = ws.split();

                // Authenticate
                let auth = serde_json::to_string(&AuthMsg {
                    action: "auth",
                    key: config.api_key.clone(),
                    secret: config.api_secret.clone(),
                })?;
                if let Err(e) = write.send(Message::Text(auth)).await {
                    warn!("Failed to send auth msg: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }

                // Subscribe to bars
                let sub = serde_json::to_string(&SubscribeMsg {
                    action: "subscribe",
                    bars: symbols.clone(),
                })?;
                if let Err(e) = write.send(Message::Text(sub)).await {
                    warn!("Failed to send subscribe msg: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
                info!("Subscribed to bars for: {:?}", symbols);

                // Read loop
                while let Some(msg_result) = read.next().await {
                    match msg_result {
                        Ok(Message::Text(text)) => {
                            debug!("WS recv: {}", text);
                            match serde_json::from_str::<WsMsg>(&text) {
                                Ok(WsMsg::Array(events)) => {
                                    for event in events {
                                        match event {
                                            WsEvent::Success { msg } => info!("WS auth/sub: {}", msg),
                                            WsEvent::Error { code, msg } => {
                                                error!("WS error {}: {}", code, msg);
                                                if code == 402 || code == 401 {
                                                    bail!("WebSocket auth failure {}: {}", code, msg);
                                                }
                                            }
                                            WsEvent::Bar(ws_bar) => {
                                                let bar = convert_bar(ws_bar);
                                                if tx.send(bar).await.is_err() {
                                                    info!("Bar receiver dropped — shutting down collector");
                                                    return Ok(());
                                                }
                                            }
                                            WsEvent::Subscription => {
                                                info!("Subscription confirmed");
                                            }
                                            WsEvent::Unknown => {
                                                warn!("Received unknown WS event: {}", text);
                                            }
                                        }
                                    }
                                }
                                Err(e) => warn!("Failed to parse WS message: {} — raw: {}", e, text),
                            }
                        }
                        Ok(Message::Ping(p)) => {
                            let _ = write.send(Message::Pong(p)).await;
                        }
                        Ok(Message::Close(_)) => {
                            warn!("WebSocket closed by server, reconnecting...");
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            warn!("WebSocket read error: {}, will reconnect...", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                warn!("WebSocket connect failed: {}. Retrying in {}s...", e, backoff_secs);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(30);
    }
}

fn convert_bar(ws: WsBar) -> Bar {
    Bar {
        symbol: ws.symbol,
        open: ws.open,
        high: ws.high,
        low: ws.low,
        close: ws.close,
        volume: ws.volume,
        timestamp: ws.timestamp,
    }
}
