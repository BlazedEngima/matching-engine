use crate::data::fill_type::{BookEvent, CancelEvent, InsertEvent};
use crate::data::order_types::IncomingSide;
use crate::data::orders::inbound_orders::{IncomingLimitOrder, IncomingMarketOrder};
use crate::data::orders::resting_orders::{OrderId, RestingOrder};
use crate::orderbook::util::book_side::BookSide;
use crate::orderbook::util::match_iter::MatchIter;
use crate::orderbook::util::price_key::PriceKey;
use crate::orderbook::util::side::{Asks, Bids};

use chrono::Utc;
use rustc_hash::{FxBuildHasher, FxHashMap};
use slab::Slab;
use std::cmp::Reverse;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
    pub fn new(capacity: usize) -> Self {
        Self {
            bids: BookSide::default(),
            asks: BookSide::default(),
            orders: Slab::with_capacity(capacity),
            order_map: FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher),
        }
    }

    /// Wrapper for insert_order
    #[inline(always)]
    pub fn insert_bids<T: Into<RestingOrder>>(&mut self, order: T, remaining: u32) -> BookEvent {
        self.insert_order::<true, _>(order, remaining)
    }

    #[inline(always)]
    pub fn insert_asks<T: Into<RestingOrder>>(&mut self, order: T, remaining: u32) -> BookEvent {
        self.insert_order::<false, _>(order, remaining)
    }

    /// Insert new order into order book
    pub fn insert_order<const IS_BID: bool, T: Into<RestingOrder>>(
        &mut self,
        order: T,
        remaining: u32,
    ) -> BookEvent {
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

        BookEvent::Insert(InsertEvent {
            order_id: self.orders[idx].order_id,
            price,
            qty: remaining,
            ts: Utc::now().timestamp_micros(),
        })
    }

    /// Cancel an existing order by OrderId
    /// Will do nothing if order doesn't exist
    pub fn cancel_order(&mut self, order_id: OrderId) -> Vec<BookEvent> {
        let idx = match self.order_map.remove(&order_id) {
            Some(i) => i,
            None => {
                // TODO: Better logging here
                println!("Order id not found during cancel");
                return vec![];
            }
        };

        let price_key = PriceKey(self.orders[idx].price);
        let qty = self.orders[idx].qty;

        let side = self.orders[idx].side.clone();
        let level = match side {
            IncomingSide::Buy => self.bids.level_mut(Reverse(price_key.clone())),
            IncomingSide::Sell => self.asks.level_mut(price_key.clone()),
        };

        let prev = self.orders[idx].prev;
        let next = self.orders[idx].next;

        // Update FIFO
        if let Some(prev_idx) = prev {
            self.orders[prev_idx].next = next;
        } else {
            level.head = next;
        }

        level.total_orders -= 1;

        if level.head.is_none() {
            match side {
                IncomingSide::Buy => {
                    self.bids.levels.remove(&std::cmp::Reverse(price_key));
                }
                IncomingSide::Sell => {
                    self.asks.levels.remove(&price_key);
                }
            }
        }

        vec![BookEvent::Cancel(CancelEvent {
            order_id,
            qty,
            ts: Utc::now().timestamp_micros(),
        })]
    }

    #[inline]
    pub fn match_market_buy(&mut self, order: &IncomingMarketOrder) -> MatchIter<'_, Asks> {
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
    pub fn match_market_sell(&mut self, order: &IncomingMarketOrder) -> MatchIter<'_, Bids> {
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
    pub fn match_limit_buy(&mut self, order: &IncomingLimitOrder) -> MatchIter<'_, Asks> {
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
    pub fn match_limit_sell(&mut self, order: &IncomingLimitOrder) -> MatchIter<'_, Bids> {
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

    /// For checking equality of order book state via a checksum
    pub fn checksum(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Bids
        for (price, level) in &self.bids.levels {
            price.hash(&mut hasher);
            level.total_orders.hash(&mut hasher);

            let mut current = level.head;

            while let Some(idx) = current {
                let order = &self.orders[idx];

                // Hash logical order state only
                order.order_id.hash(&mut hasher);
                order.qty.hash(&mut hasher);
                order.side.hash(&mut hasher);

                current = order.next;
            }
        }

        // Asks
        for (price, level) in &self.asks.levels {
            price.hash(&mut hasher);
            level.total_orders.hash(&mut hasher);

            let mut current = level.head;

            while let Some(idx) = current {
                let order = &self.orders[idx];

                // Hash logical order state only
                order.order_id.hash(&mut hasher);
                order.qty.hash(&mut hasher);
                order.side.hash(&mut hasher);

                current = order.next;
            }
        }

        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::data::fill_type::MatchEvent;

    use super::*;

    fn resting(id: u64, price: u64, qty: u32, side: IncomingSide) -> RestingOrder {
        RestingOrder {
            order_id: id,
            price,
            qty,
            side,
            next: None,
            prev: None,
            ts: Utc::now().timestamp_micros(),
        }
    }

    fn market(id: u64, qty: u32, side: IncomingSide) -> IncomingMarketOrder {
        IncomingMarketOrder {
            order_id: id,
            qty,
            side,
        }
    }

    fn limit(id: u64, price: u64, qty: u32, side: IncomingSide) -> IncomingLimitOrder {
        IncomingLimitOrder {
            order_id: id,
            price,
            qty,
            side,
        }
    }

    fn match_event(e: &BookEvent) -> &MatchEvent {
        match e {
            BookEvent::Match(fill) => fill,
            _ => panic!("Expected MatchEvent"),
        }
    }

    fn assert_book_consistency(book: &OrderBook) {
        for level in book.bids.levels.values() {
            assert!(level.head.is_some());
            assert!(level.total_orders > 0);
        }
        for level in book.asks.levels.values() {
            assert!(level.head.is_some());
            assert!(level.total_orders > 0);
        }
    }

    #[test]
    fn test_price_sorting() {
        let mut book = OrderBook::default();

        book.insert_bids(resting(1, 100, 5, IncomingSide::Buy), 5);
        book.insert_bids(resting(2, 105, 5, IncomingSide::Buy), 5);
        book.insert_bids(resting(3, 102, 5, IncomingSide::Buy), 5);

        book.insert_asks(resting(4, 110, 5, IncomingSide::Sell), 5);
        book.insert_asks(resting(5, 108, 5, IncomingSide::Sell), 5);
        book.insert_asks(resting(6, 115, 5, IncomingSide::Sell), 5);

        // Best bid should be highest
        let best_bid = book.bids.levels.first_key_value().unwrap().0;
        assert_eq!(best_bid.0.0, 105);

        // Best ask should be lowest
        let best_ask = book.asks.levels.first_key_value().unwrap().0;
        assert_eq!(best_ask.0, 108);
        assert_book_consistency(&book);
    }

    #[test]
    fn test_fifo_same_price() {
        let mut book = OrderBook::default();

        book.insert_asks(resting(1, 100, 5, IncomingSide::Sell), 5);
        book.insert_asks(resting(2, 100, 5, IncomingSide::Sell), 5);
        book.insert_asks(resting(3, 100, 5, IncomingSide::Sell), 5);

        let level = book.asks.levels.get(&PriceKey(100)).unwrap();

        let head = level.head.unwrap();
        let second = book.orders[head].next.unwrap();
        let third = book.orders[second].next.unwrap();

        assert_eq!(book.orders[head].order_id, 1);
        assert_eq!(book.orders[second].order_id, 2);
        assert_eq!(book.orders[third].order_id, 3);
        assert_book_consistency(&book);
    }

    #[test]
    fn test_cancel_removes_order_and_level() {
        let mut book = OrderBook::default();

        book.insert_bids(resting(1, 100, 10, IncomingSide::Buy), 10);

        // Level should exist
        assert_eq!(book.bids.levels.len(), 1);

        book.cancel_order(1);

        // After cancel, price level should be gone
        assert!(book.bids.levels.is_empty());
        assert_book_consistency(&book);
    }

    #[test]
    fn test_match_price_time_priority() {
        let mut book = OrderBook::default();

        // Bids
        book.insert_bids(resting(3, 101, 5, IncomingSide::Buy), 5);
        book.insert_bids(resting(1, 102, 5, IncomingSide::Buy), 5);
        book.insert_bids(resting(2, 102, 5, IncomingSide::Buy), 5);

        book.bids.print_levels();

        let mut iter = book.match_limit_sell(&limit(4, 101, 8, IncomingSide::Sell));

        let fills: Vec<_> = iter.by_ref().collect();

        assert_eq!(fills.len(), 2);

        // FIFO within price 100
        assert_eq!(match_event(&fills[0]).maker, 1);
        assert_eq!(match_event(&fills[0]).taker, 4);
        assert_eq!(match_event(&fills[0]).price, 102);
        assert_eq!(match_event(&fills[0]).qty, 5);

        assert_eq!(match_event(&fills[1]).maker, 2);
        assert_eq!(match_event(&fills[1]).taker, 4);
        assert_eq!(match_event(&fills[1]).price, 102);
        assert_eq!(match_event(&fills[1]).qty, 3);

        assert_eq!(book.bids.levels.len(), 2);
        assert_eq!(book.orders.len(), 2);

        assert_book_consistency(&book);
    }

    #[test]
    fn test_price_level_removed_after_full_fill() {
        let mut book = OrderBook::default();

        book.insert_asks(resting(1, 100, 5, IncomingSide::Sell), 5);
        book.insert_asks(resting(2, 100, 5, IncomingSide::Sell), 5);
        book.insert_asks(resting(3, 101, 6, IncomingSide::Sell), 6);

        let mut iter = book.match_limit_buy(&limit(4, 101, 16, IncomingSide::Buy));
        let fills: Vec<_> = iter.by_ref().collect();

        assert_eq!(fills.len(), 3);

        // FIFO within price 100
        assert_eq!(match_event(&fills[0]).maker, 1);
        assert_eq!(match_event(&fills[0]).taker, 4);
        assert_eq!(match_event(&fills[0]).price, 100);
        assert_eq!(match_event(&fills[0]).qty, 5);

        assert_eq!(match_event(&fills[1]).maker, 2);
        assert_eq!(match_event(&fills[1]).taker, 4);
        assert_eq!(match_event(&fills[1]).price, 100);
        assert_eq!(match_event(&fills[1]).qty, 5);

        assert_eq!(match_event(&fills[2]).maker, 3);
        assert_eq!(match_event(&fills[2]).taker, 4);
        assert_eq!(match_event(&fills[2]).price, 101);
        assert_eq!(match_event(&fills[2]).qty, 6);

        assert!(book.asks.levels.is_empty());
    }

    #[test]
    fn test_limit_order_no_match() {
        let mut book = OrderBook::default();

        // Existing ask at 105
        book.insert_asks(resting(1, 105, 5, IncomingSide::Sell), 5);

        let mut iter = book.match_limit_buy(&limit(999, 100, 5, IncomingSide::Buy));

        let fills: Vec<_> = iter.by_ref().collect();

        assert!(fills.is_empty());

        // Entire quantity should remain
        let remaining = iter.remaining();
        assert_eq!(remaining, 5);

        // Simulate typical behavior: remainder inserted
        book.insert_bids(resting(999, 100, 5, IncomingSide::Buy), 5);

        let best_bid = book.best_bid().unwrap();
        assert_eq!(best_bid.0, PriceKey(100));

        let level = book
            .bids
            .levels
            .get(&std::cmp::Reverse(PriceKey(100)))
            .unwrap();

        assert_eq!(level.total_orders, 1);

        let head_idx = level.head.unwrap();
        assert_eq!(book.orders[head_idx].order_id, 999);
        assert_eq!(book.orders[head_idx].qty, 5);
    }

    #[test]
    fn test_market_partial_and_full_fill() {
        let mut book = OrderBook::default();

        // Add asks
        book.insert_asks(resting(1, 100, 5, IncomingSide::Sell), 5);
        book.insert_asks(resting(2, 100, 5, IncomingSide::Sell), 5);
        book.insert_asks(resting(3, 101, 10, IncomingSide::Sell), 10);

        // Market buy for qty 8
        let fills: Vec<_> = book
            .match_market_buy(&market(4, 8, IncomingSide::Buy))
            .collect();

        // Should consume:
        // Order 1 (5)
        // Order 2 (3 partial)
        assert_eq!(fills.len(), 2);

        assert_eq!(match_event(&fills[0]).maker, 1);
        assert_eq!(match_event(&fills[0]).price, 100);
        assert_eq!(match_event(&fills[0]).qty, 5);

        assert_eq!(match_event(&fills[1]).maker, 2);
        assert_eq!(match_event(&fills[1]).price, 100);
        assert_eq!(match_event(&fills[1]).qty, 3);

        // Order 2 should still have 2 remaining
        let remaining = book.get_order(2).unwrap();
        assert_eq!(remaining.qty, 2);

        // Price level 100 should still exist
        assert_eq!(book.asks.levels.len(), 2);
        assert_book_consistency(&book);
    }

    #[test]
    fn test_full_price_level_cleanup_after_market_match() {
        let mut book = OrderBook::default();

        book.insert_asks(resting(1, 100, 5, IncomingSide::Sell), 5);

        // Fully consume
        let fills: Vec<_> = book
            .match_market_buy(&market(2, 5, IncomingSide::Buy))
            .collect();

        assert_eq!(fills.len(), 1);
        assert!(book.asks.levels.is_empty());
        assert_book_consistency(&book);
    }

    #[test]
    fn test_multi_price_level_match_market_order() {
        let mut book = OrderBook::default();

        book.insert_asks(resting(1, 100, 5, IncomingSide::Sell), 5);
        book.insert_asks(resting(2, 101, 5, IncomingSide::Sell), 5);
        book.insert_asks(resting(3, 102, 5, IncomingSide::Sell), 5);

        let fills: Vec<_> = book
            .match_market_buy(&market(4, 12, IncomingSide::Buy))
            .collect();

        // Should match strictly price-time priority:
        // 100 -> 101 -> 102
        assert_eq!(fills.len(), 3);

        assert_eq!(match_event(&fills[0]).price, 100);
        assert_eq!(match_event(&fills[0]).qty, 5);
        assert_eq!(match_event(&fills[1]).price, 101);
        assert_eq!(match_event(&fills[1]).qty, 5);
        assert_eq!(match_event(&fills[2]).price, 102);
        assert_eq!(match_event(&fills[2]).qty, 2);
        assert_book_consistency(&book);
    }
}
