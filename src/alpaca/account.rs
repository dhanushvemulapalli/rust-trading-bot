//! Alpaca account REST endpoints.

use crate::alpaca::client::AlpacaClient;
use anyhow::Result;
use serde::Deserialize;

/// Raw account response from Alpaca.
#[derive(Debug, Deserialize)]
pub struct AlpacaAccount {
    pub id: String,
    pub account_number: String,
    pub status: String,
    pub currency: String,
    pub cash: String,
    pub portfolio_value: String,
    pub pattern_day_trader: bool,
    pub trading_blocked: bool,
    pub transfers_blocked: bool,
    pub account_blocked: bool,
    pub buying_power: String,
    pub long_market_value: String,
    pub short_market_value: String,
    pub equity: String,
    pub last_equity: String,
    pub multiplier: String,
    pub daytrade_count: i64,
    pub daytrading_buying_power: String,
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
