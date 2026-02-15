use std::fmt;

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

impl fmt::Display for IncomingSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IncomingSide::Buy => write!(f, "BUY"),
            IncomingSide::Sell => write!(f, "SELL"),
        }
    }
}
