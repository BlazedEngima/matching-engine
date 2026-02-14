use chrono::Utc;

use crate::data::orders::inbound_orders::{IncomingLimitOrder, IncomingMarketOrder};

pub type OrderId = u64;

pub struct RestingOrder {
    pub order_id: OrderId,
    pub price: u64,
    pub qty: u32,
    pub prev: Option<usize>,
    pub next: Option<usize>,
    pub ts: i64, // microseconds since epoch
}

impl From<IncomingLimitOrder> for RestingOrder {
    fn from(order: IncomingLimitOrder) -> Self {
        Self {
            order_id: order.order_id,
            price: order.price,
            qty: order.qty,
            prev: None,
            next: None,
            ts: Utc::now().timestamp_micros(),
        }
    }
}

// Only used in edge cases where a market order
// empties out a side of the order book
impl From<IncomingMarketOrder> for RestingOrder {
    fn from(order: IncomingMarketOrder) -> Self {
        Self {
            order_id: order.order_id,
            price: 0, // Price set to 0 here, willl take on last limit price
            qty: order.qty,
            prev: None,
            next: None,
            ts: Utc::now().timestamp_micros(),
        }
    }
}
