use chrono::Utc;

use crate::data::order_types::IncomingSide;
use crate::data::orders::inbound_orders::IncomingLimitOrder;

pub type OrderId = u64;

#[derive(Debug)]
pub struct RestingOrder {
    pub order_id: OrderId,
    pub price: u64,
    pub qty: u32,
    pub side: IncomingSide,
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
            side: order.side,
            prev: None,
            next: None,
            ts: Utc::now().timestamp_micros(),
        }
    }
}
