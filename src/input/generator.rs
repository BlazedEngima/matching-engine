use crate::data::order_types::{IncomingOrder, IncomingSide};
use crate::data::orders::inbound_orders::{
    IncomingCancelOrder, IncomingLimitOrder, IncomingMarketOrder,
};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::fs::File;
use std::io::{BufWriter, Write};

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
    replay_writer: BufWriter<File>, // <-- write events to file
}

impl Generator {
    pub fn new(seed: u64, start_mid: i64, replay_path: &str) -> std::io::Result<Self> {
        let file = File::create(replay_path)?;
        Ok(Self {
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
            replay_writer: BufWriter::new(file),
        })
    }

    fn update_mid(&mut self) {
        let shift = self.rng.random_range(-self.volatility..=self.volatility);
        self.mid_price = (self.mid_price + shift).max(1);
    }

    fn write_event(&mut self, event: &IncomingOrder) {
        let line = match event {
            IncomingOrder::InboundLimit(order) => {
                format!(
                    "ADD,{},{},LIMIT,{},{}\n",
                    order.order_id,
                    match order.side {
                        IncomingSide::Buy => "B",
                        IncomingSide::Sell => "A",
                    },
                    order.price,
                    order.qty,
                )
            }
            IncomingOrder::InboundMarket(order) => {
                format!(
                    "ADD,{},{},MARKET,{}\n",
                    order.order_id,
                    match order.side {
                        IncomingSide::Buy => "B",
                        IncomingSide::Sell => "A",
                    },
                    order.qty,
                )
            }
            IncomingOrder::InboundCancel(order) => {
                format!("CANCEL,{}\n", order.order_id,)
            }
        };

        let _ = self.replay_writer.write_all(line.as_bytes());
    }

    pub fn generate(&mut self, num_events: usize) -> Vec<IncomingOrder> {
        let mut inputs = vec![];
        for _ in 0..num_events {
            self.update_mid();

            let order_id = self.next_order_id;

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
                let remove_order_id = self.active_orders.swap_remove(idx);
                let event = IncomingOrder::InboundCancel(IncomingCancelOrder {
                    order_id: remove_order_id,
                });
                self.write_event(&event);
                inputs.push(event);
                continue;
            }

            self.next_order_id += 1;

            // Generate market order
            if roll < self.market_ratio {
                let event = IncomingOrder::InboundMarket(IncomingMarketOrder {
                    order_id,
                    side,
                    qty,
                });
                self.write_event(&event);
                inputs.push(event);
                continue;
            }

            // Aggressive limit order (cross spread)
            if roll < self.market_ratio + self.cross_ratio {
                let price = match side {
                    IncomingSide::Buy => self.mid_price + self.spread,
                    IncomingSide::Sell => self.mid_price - self.spread,
                };

                self.active_orders.push(order_id);

                let event = IncomingOrder::InboundLimit(IncomingLimitOrder {
                    order_id,
                    side,
                    price: price as u64,
                    qty,
                });

                self.write_event(&event);
                inputs.push(event);
                continue;
            }

            // Passive limit order (around mid price)
            let distance = self.rng.random_range(0..=5);

            let price = match side {
                IncomingSide::Buy => self.mid_price - distance,
                IncomingSide::Sell => self.mid_price + distance,
            }
            .max(1);

            self.active_orders.push(order_id);

            let event = IncomingOrder::InboundLimit(IncomingLimitOrder {
                order_id,
                side,
                price: price as u64,
                qty,
            });

            self.write_event(&event);
            inputs.push(event);
        }

        inputs
    }
}
