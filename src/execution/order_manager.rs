//! Order manager — translates approved orders into Alpaca API calls
//! and maintains in-memory order state.

use crate::alpaca::{
    client::AlpacaClient,
    orders::{OrderRequest, OrderSide, OrderType, TimeInForce},
};
use crate::risk::manager::ApprovedOrder;
use crate::strategy::Signal;
use anyhow::Result;
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Internal state of a tracked order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Submitted,
    Accepted,
    PartiallyFilled { filled_qty: u64 },
    Filled,
    Canceled,
    Expired,
    Rejected,
    Failed,
}

/// Our internal view of an order (separate from Alpaca's raw type).
#[derive(Debug, Clone)]
pub struct ManagedOrder {
    pub id: Uuid,
    pub client_order_id: String,
    pub symbol: String,
    pub side: Signal,
    pub requested_qty: f64,
    pub filled_qty: f64,
    pub status: OrderStatus,
}

/// Manages order submission and lifecycle state.
pub struct OrderManager {
    client: AlpacaClient,
    /// All orders submitted this session, keyed by Alpaca order ID.
    orders: HashMap<Uuid, ManagedOrder>,
}

impl OrderManager {
    pub fn new(client: AlpacaClient) -> Self {
        Self {
            client,
            orders: HashMap::new(),
        }
    }

    /// Submit an approved order to Alpaca.
    ///
    /// Does nothing if the approved order is for a Hold signal.
    pub async fn submit(&mut self, approved: &ApprovedOrder) -> Result<Option<Uuid>> {
        if approved.signal == Signal::Hold {
            return Ok(None);
        }

        let side = match approved.signal {
            Signal::Buy => OrderSide::Buy,
            Signal::Sell => OrderSide::Sell,
            Signal::Hold => unreachable!(),
        };

        let client_order_id = format!("tbot-{}", Uuid::new_v4());

        let req = OrderRequest {
            symbol: approved.symbol.clone(),
            qty: format!("{}", approved.qty as u64),
            side,
            r#type: OrderType::Market,
            time_in_force: TimeInForce::Day,
            limit_price: None,
            stop_price: None,
            client_order_id: Some(client_order_id.clone()),
        };

        match self.client.submit_order(req).await {
            Ok(order) => {
                info!(
                    order_id = %order.id,
                    symbol = %approved.symbol,
                    signal = ?approved.signal,
                    qty = %approved.qty,
                    "Order submitted to Alpaca"
                );
                let managed = ManagedOrder {
                    id: order.id,
                    client_order_id: client_order_id.clone(),
                    symbol: approved.symbol.clone(),
                    side: approved.signal,
                    requested_qty: approved.qty,
                    filled_qty: 0.0,
                    status: OrderStatus::Submitted,
                };
                self.orders.insert(order.id, managed);
                Ok(Some(order.id))
            }
            Err(e) => {
                error!(
                    symbol = %approved.symbol,
                    error = %e,
                    "Failed to submit order to Alpaca"
                );
                Err(e)
            }
        }
    }

    /// Update the status of a managed order from a trade-update event.
    pub fn on_order_update(&mut self, order_id: Uuid, new_status: OrderStatus, filled_qty: f64) {
        if let Some(order) = self.orders.get_mut(&order_id) {
            info!(
                order_id = %order_id,
                old_status = ?order.status,
                new_status = ?new_status,
                filled_qty = %filled_qty,
                "Order status updated"
            );
            order.status = new_status;
            order.filled_qty = filled_qty;
        } else {
            warn!(order_id = %order_id, "Received update for unknown order");
        }
    }

    /// Get a snapshot of all managed orders.
    pub fn orders(&self) -> &HashMap<Uuid, ManagedOrder> {
        &self.orders
    }
}
