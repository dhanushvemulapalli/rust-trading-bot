//! Alpaca order REST endpoints and WebSocket trade-update handling.

use crate::alpaca::client::AlpacaClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Request types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub qty: String,
    pub side: OrderSide,
    pub r#type: OrderType,
    pub time_in_force: TimeInForce,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeInForce {
    Day,
    Gtc,
    Opg,
    Ioc,
    Fok,
}

// ── Response types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct AlpacaOrder {
    pub id: Uuid,
    pub client_order_id: String,
    pub status: String,
    pub symbol: String,
    pub qty: String,
    pub filled_qty: String,
    pub side: OrderSide,
    pub r#type: OrderType,
    pub time_in_force: TimeInForce,
    pub limit_price: Option<String>,
    pub stop_price: Option<String>,
    pub filled_avg_price: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub submitted_at: Option<String>,
    pub filled_at: Option<String>,
    pub expired_at: Option<String>,
    pub canceled_at: Option<String>,
    pub failed_at: Option<String>,
}

// ── AlpacaClient methods ───────────────────────────────────────────────────────

impl AlpacaClient {
    /// Submit a new order.
    pub async fn submit_order(&self, req: OrderRequest) -> Result<AlpacaOrder> {
        let url = format!("{}/v2/orders", self.base_url);
        let order = self
            .http
            .post(&url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json::<AlpacaOrder>()
            .await?;
        Ok(order)
    }

    /// Cancel an order by its Alpaca order ID.
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<()> {
        let url = format!("{}/v2/orders/{}", self.base_url, order_id);
        self.http
            .delete(&url)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Fetch all open orders.
    pub async fn list_open_orders(&self) -> Result<Vec<AlpacaOrder>> {
        let url = format!("{}/v2/orders?status=open", self.base_url);
        let orders = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<AlpacaOrder>>()
            .await?;
        Ok(orders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_side_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&OrderSide::Buy).unwrap(),
            r#""buy""#
        );
        assert_eq!(
            serde_json::to_string(&OrderSide::Sell).unwrap(),
            r#""sell""#
        );
    }
}
