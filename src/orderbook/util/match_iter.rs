use crate::data::fill_type::FillEvent;
use crate::data::orders::resting_orders::{OrderId, RestingOrder};
use crate::orderbook::util::book_side::BookSide;
use crate::orderbook::util::side::Side;
use chrono::Utc;
use rustc_hash::FxHashMap;
use slab::Slab;

pub struct MatchIter<'a, OrderSide: Side> {
    side: &'a mut BookSide<OrderSide>,
    orders: &'a mut Slab<RestingOrder>,
    order_map: &'a mut FxHashMap<OrderId, usize>,
    order_id: u64,
    remaining: u32,
    price_limit: Option<OrderSide::Key>,
}

impl<'a, OrderSide: Side> MatchIter<'a, OrderSide> {
    pub fn new(
        side: &'a mut BookSide<OrderSide>,
        orders: &'a mut Slab<RestingOrder>,
        order_map: &'a mut FxHashMap<OrderId, usize>,
        order_id: u64,
        remaining: u32,
        price_limit: Option<OrderSide::Key>,
    ) -> Self {
        Self {
            side,
            orders,
            order_map,
            order_id,
            remaining,
            price_limit,
        }
    }
}

impl<'a, OrderSide: Side> Iterator for MatchIter<'a, OrderSide> {
    type Item = FillEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        // Best price level
        let mut entry = self.side.levels.first_entry()?;

        if let Some(price_limit) = &self.price_limit
            && OrderSide::compare_price(entry.key(), price_limit)
        {
            return None;
        }

        let best_price = OrderSide::key_to_price(entry.key().clone());
        let level = entry.get_mut();

        let slab_index = level.head?;

        let qty = self.orders[slab_index].qty;

        let traded = self.remaining.min(qty);
        self.orders[slab_index].qty -= traded;
        self.remaining -= traded;

        let order_id = self.orders[slab_index].order_id;

        // If fully filled
        if self.orders[slab_index].qty == 0 {
            let next = self.orders[slab_index].next;

            // Advance linked list
            level.head = next;

            if next.is_none() {
                level.tail = None;
            }

            level.total_orders -= 1;

            // Remove from slab + map
            self.orders.remove(slab_index);
            self.order_map.remove(&order_id);
        }

        // If price level empty -> remove it
        if level.head.is_none() {
            entry.remove();
        }

        Some(FillEvent {
            maker: order_id,
            taker: self.order_id,
            price: best_price.0,
            qty: traded,
            ts: Utc::now().timestamp_micros(),
        })
    }
}
