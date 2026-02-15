use crate::data::{order_types::IncomingSide, orders::resting_orders::OrderId};

pub enum BookEvent {
    Match(MatchEvent),
    Cancel(CancelEvent),
    Insert(InsertEvent),
    BookSnapshot(String),
}

pub struct MatchEvent {
    pub maker: OrderId,
    pub taker: OrderId,
    pub price: u64,
    pub qty: u32,
    pub ts: i64,
}

pub struct CancelEvent {
    pub order_id: OrderId,
    pub qty: u32,
    pub ts: i64,
}

pub struct InsertEvent {
    pub order_id: OrderId,
    pub price: u64,
    pub side: IncomingSide,
    pub qty: u32,
    pub ts: i64,
}
