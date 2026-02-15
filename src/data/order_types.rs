use crate::data::orders::inbound_orders::{
    IncomingCancelOrder, IncomingLimitOrder, IncomingMarketOrder,
};

#[repr(u8)]
#[derive(Debug, Clone, Hash)]
pub enum IncomingSide {
    Buy = 0,
    Sell = 1,
}

#[derive(Debug)]
pub enum IncomingOrder {
    InboundLimit(IncomingLimitOrder),
    InboundMarket(IncomingMarketOrder),
    InboundCancel(IncomingCancelOrder),
}
