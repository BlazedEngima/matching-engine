use crate::data::order_types::{IncomingOrder, IncomingSide};
use crate::data::orders::inbound_orders::{
    IncomingCancelOrder, IncomingLimitOrder, IncomingMarketOrder,
};
use crate::input::traits::EventSource;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

pub struct Generator {
    rng: StdRng,
    next_order_id: u64,
    mid_price: i64,
    spread: i64,
    volatility: i64,
    cross_ratio: f64,
    market_ratio: f64,
    cancel_ratio: f64,
    max_qty: u32,
    active_orders: Vec<u64>,
}

impl Generator {
    pub fn new(seed: u64, start_mid: i64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            next_order_id: 1,
            mid_price: start_mid,
            spread: 5,
            volatility: 1,
            cross_ratio: 0.4,
            market_ratio: 0.25,
            cancel_ratio: 0.1,
            max_qty: 1 << 20,
            active_orders: Vec::new(),
        }
    }

    fn update_mid(&mut self) {
        let shift = self.rng.random_range(-self.volatility..=self.volatility);
        self.mid_price = (self.mid_price + shift).max(1);
    }
}

impl EventSource for Generator {
    fn next_event(&mut self) -> Option<IncomingOrder> {
        self.update_mid();

        let order_id = self.next_order_id;
        self.next_order_id += 1;

        let side = if self.rng.random_bool(0.5) {
            IncomingSide::Buy
        } else {
            IncomingSide::Sell
        };

        let qty = self.rng.random_range(1..=self.max_qty);

        let roll: f64 = self.rng.random_range(0.0..1.0);

        // Generate cancel order
        if roll < self.cancel_ratio && !self.active_orders.is_empty() {
            let idx = self.rng.random_range(0..self.active_orders.len());
            let order_id = self.active_orders.swap_remove(idx);
            return Some(IncomingOrder::InboundCancel(IncomingCancelOrder {
                order_id,
            }));
        }

        // Generate market order
        if roll < self.market_ratio {
            return Some(IncomingOrder::InboundMarket(IncomingMarketOrder {
                order_id,
                side,
                qty,
            }));
        }

        // Aggressive limit order (cross spread)
        if roll < self.market_ratio + self.cross_ratio {
            let price = match side {
                IncomingSide::Buy => self.mid_price + self.spread,
                IncomingSide::Sell => self.mid_price - self.spread,
            };

            self.active_orders.push(order_id);

            return Some(IncomingOrder::InboundLimit(IncomingLimitOrder {
                order_id,
                side,
                price: price as u64,
                qty,
            }));
        }

        // Passive limit order (around mid price)
        let distance = self.rng.random_range(0..=5);

        let price = match side {
            IncomingSide::Buy => self.mid_price - distance,
            IncomingSide::Sell => self.mid_price + distance,
        }
        .max(1);

        self.active_orders.push(order_id);

        Some(IncomingOrder::InboundLimit(IncomingLimitOrder {
            order_id,
            side,
            price: price as u64,
            qty,
        }))
    }
}
