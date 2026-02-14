use crate::data::order_types::IncomingSide;

#[allow(dead_code)]
pub struct IncomingLimitOrder {
    pub order_id: u64, // u64 for simplicity. Probably use UUID in real scenarios.
    pub price: u64,
    pub qty: u32,
    pub side: IncomingSide,
}

#[allow(dead_code)]
pub struct IncomingMarketOrder {
    pub order_id: u64,
    pub qty: u32,
    pub side: IncomingSide,
}

#[allow(dead_code)]
pub struct IncomingCancelOrder {
    pub order_id: u64,
}
