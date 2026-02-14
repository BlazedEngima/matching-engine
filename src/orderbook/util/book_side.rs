use crate::data::price_level::PriceLevel;
use crate::orderbook::util::side::Side;
use std::collections::BTreeMap;
use std::io::{BufWriter, Write};

#[derive(Debug)]
pub struct BookSide<OrderSide: Side> {
    pub levels: BTreeMap<OrderSide::Key, PriceLevel>,
}

impl<OrderSide: Side> Default for BookSide<OrderSide> {
    fn default() -> Self {
        Self {
            levels: BTreeMap::default(),
        }
    }
}

impl<OrderSide: Side> BookSide<OrderSide> {
    pub fn level_mut<T: Into<OrderSide::Key>>(&mut self, price: T) -> &mut PriceLevel {
        self.levels.entry(price.into()).or_default()
    }

    // For testing mainly
    pub fn print_levels<W: Write>(&self, output: &mut BufWriter<W>) -> std::io::Result<()> {
        for (key, level) in &self.levels {
            let price = OrderSide::key_to_price(key.clone());
            output.write_all(
                format!("Price: {:?} | Orders: {}\n", price, level.total_orders).as_bytes(),
            )?;
        }

        Ok(())
    }
}
