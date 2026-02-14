use crate::data::order_types::{IncomingOrder, IncomingSide};
use crate::data::orders::inbound_orders::{
    IncomingCancelOrder, IncomingLimitOrder, IncomingMarketOrder,
};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct ReplayReader<R: BufRead> {
    reader: R,
    buffer: String,
}

impl ReplayReader<BufReader<File>> {
    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::new(file),
            buffer: String::with_capacity(256),
        })
    }

    pub fn parse_orders(&mut self) -> Vec<IncomingOrder> {
        let mut inputs = vec![];
        while self
            .reader
            .read_line(&mut self.buffer)
            .expect("Unable to open replay buffer")
            != 0
        {
            match parse_event(&self.buffer) {
                Some(event) => inputs.push(event),
                _ => {
                    println!("Error: Invalid input in replay file");
                }
            };

            self.buffer.clear(); // Important: clear buffer for next read
        }

        inputs
    }
}

fn parse_event(line: &str) -> Option<IncomingOrder> {
    let mut parts = line.split(',');

    match parts.next()? {
        "ADD" => {
            let order_id = parts.next()?.parse().ok()?;

            let side = match parts.next()? {
                "B" => IncomingSide::Buy,
                "A" => IncomingSide::Sell,
                _ => return None,
            };

            match parts.next()? {
                "LIMIT" => {
                    let price = parts.next()?.parse().ok()?;
                    let qty = parts.next()?.parse().ok()?;

                    Some(IncomingOrder::InboundLimit(IncomingLimitOrder {
                        order_id,
                        side,
                        price,
                        qty,
                    }))
                }
                "MARKET" => {
                    let qty = parts.next()?.parse().ok()?;

                    Some(IncomingOrder::InboundMarket(IncomingMarketOrder {
                        order_id,
                        side,
                        qty,
                    }))
                }
                _ => None,
            }
        }

        "CANCEL" => {
            let order_id = parts.next()?.parse().ok()?;

            Some(IncomingOrder::InboundCancel(IncomingCancelOrder {
                order_id,
            }))
        }

        _ => None,
    }
}
