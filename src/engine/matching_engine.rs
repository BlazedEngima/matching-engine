use crate::data::fill_type::FillEvent;
use crate::data::order_types::{InboundOrderType, IncomingSide};
use crate::data::orders::inbound_orders::{
    IncomingCancelOrder, IncomingLimitOrder, IncomingMarketOrder,
};
use crate::orderbook::order_book::OrderBook;

pub struct Engine {
    book: OrderBook,
}

impl Engine {
    pub fn match_order(&mut self, order: InboundOrderType) -> Vec<FillEvent> {
        match order {
            InboundOrderType::InboundLimit(limit) => self.match_limit(limit),
            InboundOrderType::InboundMarket(market) => self.match_market(market),
            InboundOrderType::InboundCancel(cancel) => self.match_cancel(cancel),
        }
    }

    pub fn match_limit(&mut self, order: IncomingLimitOrder) -> Vec<FillEvent> {
        match order.side {
            IncomingSide::Buy => {
                let mut iter = self.book.match_limit_buy(&order);
                let fill = iter.by_ref().collect();
                let remaining = iter.remaining();

                if remaining > 0 {
                    self.book.insert_bids(order, remaining);
                }

                fill
            }

            IncomingSide::Sell => {
                let mut iter = self.book.match_limit_sell(&order);
                let fill = iter.by_ref().collect();
                let remaining = iter.remaining();

                if remaining > 0 {
                    self.book.insert_asks(order, remaining);
                }

                fill
            }
        }
    }

    pub fn match_market(&mut self, order: IncomingMarketOrder) -> Vec<FillEvent> {
        match order.side {
            IncomingSide::Buy => {
                let mut iter = self.book.match_market_buy(&order);
                iter.by_ref().collect()
            }

            IncomingSide::Sell => {
                let mut iter = self.book.match_market_sell(&order);
                iter.by_ref().collect()
            }
        }
    }

    pub fn match_cancel(&mut self, order: IncomingCancelOrder) -> Vec<FillEvent> {
        self.book.cancel_order(order.order_id);
        vec![]
    }
}
