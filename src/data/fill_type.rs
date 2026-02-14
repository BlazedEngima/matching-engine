use crate::data::orders::resting_orders::OrderId;

pub struct Fill {
    pub maker: OrderId,
    pub taker: OrderId,
    pub price: u64,
    pub qty: u32,
    pub ts: i64,
}
