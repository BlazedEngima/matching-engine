use crate::data::book_event::BookEvent;
use crate::data::order_types::{IncomingOrder, IncomingSide};
use crate::data::orders::inbound_orders::{
    IncomingCancelOrder, IncomingLimitOrder, IncomingMarketOrder,
};
use crate::orderbook::order_book::OrderBook;

#[derive(Default)]
pub struct Engine {
    book: OrderBook,
}

impl Engine {
    pub fn new(capacity: usize) -> Self {
        Self {
            book: OrderBook::new(capacity),
        }
    }
    pub fn match_order(&mut self, order: IncomingOrder) -> Vec<BookEvent> {
        match order {
            IncomingOrder::InboundLimit(limit) => self.match_limit(limit),
            IncomingOrder::InboundMarket(market) => self.match_market(market),
            IncomingOrder::InboundCancel(cancel) => self.match_cancel(cancel),
        }
    }

    pub fn match_limit(&mut self, order: IncomingLimitOrder) -> Vec<BookEvent> {
        match order.side {
            IncomingSide::Buy => {
                let mut iter = self.book.match_limit_buy(&order);
                let mut fill: Vec<BookEvent> = iter.by_ref().collect();
                let remaining = iter.remaining();

                if remaining > 0 {
                    fill.push(self.book.insert_bids(order, remaining));
                }

                fill
            }

            IncomingSide::Sell => {
                let mut iter = self.book.match_limit_sell(&order);
                let mut fill: Vec<BookEvent> = iter.by_ref().collect();
                let remaining = iter.remaining();

                if remaining > 0 {
                    fill.push(self.book.insert_asks(order, remaining));
                }

                fill
            }
        }
    }

    pub fn match_market(&mut self, order: IncomingMarketOrder) -> Vec<BookEvent> {
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

    pub fn match_cancel(&mut self, order: IncomingCancelOrder) -> Vec<BookEvent> {
        self.book.cancel_order(order.order_id)
    }

    #[inline]
    pub fn get_book(&self) -> &OrderBook {
        &self.book
    }
}
