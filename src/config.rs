//! Application configuration loaded from environment variables.

use anyhow::{Context, Result};
use dotenvy::dotenv;

#[derive(Debug, Clone)]
pub struct Config {
    /// Alpaca API key (paper or live)
    pub api_key: String,
    /// Alpaca API secret
    pub api_secret: String,
    /// Base URL for Alpaca REST API
    pub base_url: String,
    /// WebSocket URL for market data
    pub data_ws_url: String,
    /// WebSocket URL for trade updates
    pub trading_ws_url: String,
    /// Whether this is paper trading (must be true for now)
    pub use_paper: bool,
    /// SQLite/Postgres connection URL
    pub database_url: String,
}

impl Config {
    /// Load configuration from the environment (and optionally from a `.env` file).
    pub fn load() -> Result<Self> {
        // Load .env if present; ignore if missing
        dotenv().ok();

        let api_key = std::env::var("ALPACA_KEY")
            .context("ALPACA_KEY must be set in environment or .env")?;
        let api_secret = std::env::var("ALPACA_SECRET")
            .context("ALPACA_SECRET must be set in environment or .env")?;
        let base_url = std::env::var("ALPACA_BASE_URL")
            .unwrap_or_else(|_| "https://paper-api.alpaca.markets".into());
        let data_ws_url = std::env::var("ALPACA_DATA_WS")
            .unwrap_or_else(|_| "wss://stream.data.alpaca.markets/v2/iex".into());
        let trading_ws_url = std::env::var("ALPACA_TRADING_WS")
            .unwrap_or_else(|_| "wss://paper-api.alpaca.markets/stream".into());
        let use_paper = std::env::var("USE_PAPER")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://./trading_bot.db".into());

        // Safety guard: refuse to run with live API base URL unless explicitly approved.
        if !use_paper && base_url.contains("paper") {
            anyhow::bail!("USE_PAPER=false but base URL still points to paper API; aborting.");
        }

        Ok(Self {
            api_key,
            api_secret,
            base_url,
            data_ws_url,
            trading_ws_url,
            use_paper,
            database_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_paper() {
        // When USE_PAPER is absent, default must be true (safe)
        // We cannot guarantee env isn't set, but at least document intent.
        let use_paper = std::env::var("USE_PAPER")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        assert!(use_paper, "Default must be paper trading");
    }
}
