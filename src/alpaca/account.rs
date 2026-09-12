//! Alpaca account REST endpoints.

use crate::alpaca::client::AlpacaClient;
use anyhow::Result;
use serde::Deserialize;

/// Raw account response from Alpaca.
#[derive(Debug, Deserialize)]
pub struct AlpacaAccount {
    pub id: String,
    #[serde(default)]
    pub account_number: String,
    pub status: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub cash: String,
    #[serde(default)]
    pub portfolio_value: String,
    #[serde(default)]
    pub pattern_day_trader: bool,
    #[serde(default)]
    pub trading_blocked: bool,
    #[serde(default)]
    pub transfers_blocked: bool,
    #[serde(default)]
    pub account_blocked: bool,
    #[serde(default)]
    pub buying_power: String,
    #[serde(default)]
    pub long_market_value: String,
    #[serde(default)]
    pub short_market_value: String,
    #[serde(default)]
    pub equity: String,
    #[serde(default)]
    pub last_equity: String,
    #[serde(default)]
    pub multiplier: String,
    #[serde(default)]
    pub daytrade_count: i64,
    #[serde(default)]
    pub daytrading_buying_power: String,
    #[serde(default)]
    pub regt_buying_power: String,
}

impl AlpacaClient {
    /// Fetch account information from Alpaca.
    pub async fn get_account(&self) -> Result<AlpacaAccount> {
        let url = format!("{}/v2/account", self.base_url);
        let account = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<AlpacaAccount>()
            .await?;
        Ok(account)
    }

    /// Fetch all open positions from Alpaca.
    pub async fn get_positions(&self) -> Result<Vec<AlpacaPosition>> {
        let url = format!("{}/v2/positions", self.base_url);
        let positions = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<AlpacaPosition>>()
            .await?;
        Ok(positions)
    }
}

/// Raw position response from Alpaca.
#[derive(Debug, Clone, Deserialize)]
pub struct AlpacaPosition {
    pub symbol: String,
    #[serde(default)]
    pub qty: String,
    #[serde(default)]
    pub side: String,
    #[serde(default)]
    pub avg_entry_price: String,
    #[serde(default)]
    pub market_value: String,
    #[serde(default)]
    pub unrealized_pl: String,
    #[serde(default)]
    pub current_price: String,
}

impl AlpacaPosition {
    pub fn to_domain(&self) -> crate::market::types::Position {
        use crate::market::types::{Position, Side};
        let side = if self.side.to_lowercase() == "short" {
            Side::Short
        } else {
            Side::Long
        };
        let qty = self.qty.parse::<f64>().unwrap_or(0.0).abs();
        let avg_entry_price = self.avg_entry_price.parse::<f64>().unwrap_or(0.0);
        let market_value = self.market_value.parse::<f64>().unwrap_or(0.0);
        let unrealized_pl = self.unrealized_pl.parse::<f64>().unwrap_or(0.0);

        Position {
            symbol: self.symbol.clone(),
            side,
            qty,
            avg_entry_price,
            market_value,
            unrealized_pl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_deserializes_from_json() {
        let json = r#"{
            "id": "acc-123",
            "account_number": "PA1234567",
            "status": "ACTIVE",
            "currency": "USD",
            "cash": "10000.00",
            "portfolio_value": "10500.00",
            "pattern_day_trader": false,
            "trading_blocked": false,
            "transfers_blocked": false,
            "account_blocked": false,
            "buying_power": "20000.00",
            "long_market_value": "500.00",
            "short_market_value": "0.00",
            "equity": "10500.00",
            "last_equity": "10000.00",
            "multiplier": "2",
            "daytrade_count": 0,
            "daytrading_buying_power": "40000.00",
            "regt_buying_power": "20000.00"
        }"#;
        let account: AlpacaAccount = serde_json::from_str(json).unwrap();
        assert_eq!(account.id, "acc-123");
        assert_eq!(account.status, "ACTIVE");
    }
}
