use crate::data::orders::inbound_orders::{
    IncomingCancelOrder, IncomingLimitOrder, IncomingMarketOrder,
};

#[repr(u8)]
pub enum IncomingSide {
    Buy = 0,
    Sell = 1,
}

pub enum InboundOrderType {
    InboundLimit(IncomingLimitOrder),
    InboundMarket(IncomingMarketOrder),
    InboundCancel(IncomingCancelOrder),
}
