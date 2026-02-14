use crate::data::orders::inbound_orders::{IncomingLimitOrder, IncomingMarketOrder};
use crate::data::orders::resting_orders::{OrderId, RestingOrder};
use crate::orderbook::util::book_side::BookSide;
use crate::orderbook::util::match_iter::MatchIter;
use crate::orderbook::util::price_key::PriceKey;
use crate::orderbook::util::side::{Asks, Bids};
use rustc_hash::{FxBuildHasher, FxHashMap};
use slab::Slab;
use std::cmp::Reverse;

pub struct OrderBook {
    bids: BookSide<Bids>,
    asks: BookSide<Asks>,

    orders: Slab<RestingOrder>,
    order_map: FxHashMap<OrderId, usize>,
}

impl Default for OrderBook {
    fn default() -> Self {
        Self {
            bids: BookSide::default(),
            asks: BookSide::default(),
            orders: Slab::with_capacity(262144),
            order_map: FxHashMap::with_capacity_and_hasher(262144, FxBuildHasher),
        }
    }
}

impl OrderBook {
    /// Insert new order into order book
    pub fn insert_order<const IS_BID: bool, T: Into<RestingOrder>>(
        &mut self,
        order: T,
        remaining: u32,
    ) {
        let mut order = order.into();
        order.qty = remaining;
        let idx = self.orders.insert(order);
        self.order_map.insert(self.orders[idx].order_id, idx);

        let price = self.orders[idx].price;
        let level = if IS_BID {
            self.bids.level_mut(Reverse(PriceKey(price)))
        } else {
            self.asks.level_mut(PriceKey(price))
        };

        // Update FIFO
        if let Some(tail) = level.tail {
            self.orders[tail].next = Some(idx);
            self.orders[idx].prev = Some(tail);
        }
        level.tail = Some(idx);
        if level.head.is_none() {
            level.head = Some(idx);
        }
        level.total_orders += 1;
    }

    /// Cancel an existing order by OrderId
    /// Will do nothing if order doesn't exist
    pub fn cancel_order<const IS_BID: bool>(&mut self, order_id: OrderId) {
        match self.order_map.remove(&order_id) {
            Some(idx) => {
                let order = self.orders.remove(idx);
                let level = if IS_BID {
                    self.bids.level_mut(Reverse(PriceKey(order.price)))
                } else {
                    self.asks.level_mut(PriceKey(order.price))
                };

                // Update FIFO
                if let Some(prev) = order.prev {
                    self.orders[prev].next = order.next;
                } else {
                    level.head = order.next;
                }
                if let Some(next) = order.next {
                    self.orders[next].prev = order.prev;
                } else {
                    level.tail = order.prev;
                }

                level.total_orders -= 1;

                // Edge case on last order in price level
                if level.head.is_none() {
                    if IS_BID {
                        self.bids.levels.remove(&Reverse(PriceKey(order.price)));
                    } else {
                        self.asks.levels.remove(&PriceKey(order.price));
                    }
                }
            }
            _ => {
                // TODO: Better logging here
                println!("Unable to find order id when cancelling. Skipping operation...");
            }
        }
    }

    #[inline]
    pub fn match_market_buy(&mut self, order: IncomingMarketOrder) -> MatchIter<'_, Asks> {
        MatchIter::new(
            &mut self.asks,
            &mut self.orders,
            &mut self.order_map,
            order.order_id,
            order.qty,
            None,
        )
    }

    #[inline]
    pub fn match_market_sell(&mut self, order: IncomingMarketOrder) -> MatchIter<'_, Bids> {
        MatchIter::new(
            &mut self.bids,
            &mut self.orders,
            &mut self.order_map,
            order.order_id,
            order.qty,
            None,
        )
    }

    #[inline]
    pub fn match_limit_buy(&mut self, order: IncomingLimitOrder) -> MatchIter<'_, Asks> {
        MatchIter::new(
            &mut self.asks,
            &mut self.orders,
            &mut self.order_map,
            order.order_id,
            order.qty,
            Some(PriceKey(order.price)),
        )
    }

    #[inline]
    pub fn match_limit_sell(&mut self, order: IncomingLimitOrder) -> MatchIter<'_, Bids> {
        MatchIter::new(
            &mut self.bids,
            &mut self.orders,
            &mut self.order_map,
            order.order_id,
            order.qty,
            Some(Reverse(PriceKey(order.price))),
        )
    }

    /// Lookup an order index by OrderId
    #[inline]
    pub fn get_index(&self, id: OrderId) -> Option<usize> {
        self.order_map.get(&id).copied()
    }

    /// Lookup order reference by OrderId
    #[inline]
    pub fn get_order(&self, id: OrderId) -> Option<&RestingOrder> {
        self.get_index(id).map(|idx| &self.orders[idx])
    }

    /// Mutable reference to order lookup by OrderId
    #[inline]
    pub fn get_order_mut(&mut self, id: OrderId) -> Option<&mut RestingOrder> {
        let idx = self.get_index(id)?;
        Some(&mut self.orders[idx])
    }

    /// Best bid price
    #[inline]
    pub fn best_bid(&self) -> Option<&Reverse<PriceKey>> {
        self.bids.levels.first_key_value().map(|(k, _)| k)
    }

    /// Best ask price
    #[inline]
    pub fn best_ask(&self) -> Option<&PriceKey> {
        self.asks.levels.first_key_value().map(|(k, _)| k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orderbook::util::side::Side;

    fn price(p: u64) -> PriceKey {
        PriceKey(p)
    }

    #[test]
    fn test_insert_and_sorting() {
        let mut book = OrderBook::default();

        // Insert bids
        book.insert_order(1, price(100), 10);
        book.insert_bid(2, price(105), 5);
        book.insert_bid(3, price(102), 7);

        // Insert asks
        book.insert_ask(4, price(110), 3);
        book.insert_ask(5, price(108), 6);
        book.insert_ask(6, price(115), 2);

        // Best bid should be highest
        let best_bid = book.bids.levels.first_key_value().unwrap();
        assert_eq!(Bids::key_to_price(best_bid.0), price(105));

        // Best ask should be lowest
        let best_ask = book.asks.levels.first_key_value().unwrap();
        assert_eq!(Asks::key_to_price(best_ask.0), price(108));
    }

    #[test]
    fn test_cancel_removes_order_and_level() {
        let mut book = OrderBook::default();

        book.insert_bid(1, price(100), 10);

        // Level should exist
        assert_eq!(book.bids.levels.len(), 1);

        book.cancel(1);

        // After cancel, price level should be gone
        assert!(book.bids.levels.is_empty());
    }

    #[test]
    fn test_matching_iterator_partial_and_full_fill() {
        let mut book = OrderBook::default();

        // Add asks
        book.insert_ask(1, price(100), 5);
        book.insert_ask(2, price(100), 5);
        book.insert_ask(3, price(101), 10);

        // Market buy for qty 8
        let fills: Vec<_> = book.match_market_buy(8).collect();

        // Should consume:
        // Order 1 (5)
        // Order 2 (3 partial)
        assert_eq!(fills.len(), 2);

        assert_eq!(fills[0].maker, 1);
        assert_eq!(fills[0].price, price(100));
        assert_eq!(fills[0].qty, 5);

        assert_eq!(fills[1].maker, 2);
        assert_eq!(fills[1].price, price(100));
        assert_eq!(fills[1].qty, 3);

        // Order 2 should still have 2 remaining
        let remaining = book.get_order(2).unwrap();
        assert_eq!(remaining.qty, 2);

        // Price level 100 should still exist
        assert_eq!(book.asks.levels.len(), 2);
    }

    #[test]
    fn test_full_price_level_cleanup_after_match() {
        let mut book = OrderBook::default();

        book.insert_ask(1, price(100), 5);

        // Fully consume
        let fills: Vec<_> = book.match_market_buy(5).collect();

        assert_eq!(fills.len(), 1);
        assert!(book.asks.levels.is_empty());
    }

    #[test]
    fn test_multi_price_level_match_order() {
        let mut book = OrderBook::default();

        book.insert_ask(1, price(100), 5);
        book.insert_ask(2, price(101), 5);
        book.insert_ask(3, price(102), 5);

        let fills: Vec<_> = book.match_market_buy(12).collect();

        // Should match strictly price-time priority:
        // 100 -> 101 -> 102
        assert_eq!(fills.len(), 3);

        assert_eq!(fills[0].price, price(100));
        assert_eq!(fills[1].price, price(101));
        assert_eq!(fills[2].price, price(102));
    }
}
