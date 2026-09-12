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
    info!("Connecting to market-data WebSocket: {}", config.data_ws_url);
    let url = url::Url::parse(&config.data_ws_url)?;
    let (ws, _) = connect_async(url).await?;
    let (mut write, mut read) = ws.split();

    // Authenticate
    let auth = serde_json::to_string(&AuthMsg {
        action: "auth",
        key: config.api_key.clone(),
        secret: config.api_secret.clone(),
    })?;
    write.send(Message::Text(auth)).await?;

    // Subscribe to bars
    let sub = serde_json::to_string(&SubscribeMsg {
        action: "subscribe",
        bars: symbols.clone(),
    })?;
    write.send(Message::Text(sub)).await?;
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
                                    bail!("WebSocket error {}: {}", code, msg);
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
                write.send(Message::Pong(p)).await?;
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket closed by server");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                error!("WebSocket read error: {}", e);
                bail!("WebSocket read error: {}", e);
            }
        }
    }

    Ok(())
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
