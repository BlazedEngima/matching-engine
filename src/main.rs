use std::cmp::Reverse;

use matching_engine::orderbook::{
    order_book::OrderBook,
    util::{
        book_side::BookSide,
        price_key::PriceKey,
        side::{Asks, Bids},
    },
};

fn main() {
    println!("--- ASK SIDE (Ascending) ---");
    let mut asks = BookSide::<Asks>::default();

    // Insert out of order intentionally
    asks.level_mut(PriceKey(99));
    asks.level_mut(PriceKey(101));
    asks.level_mut(PriceKey(103));
    asks.level_mut(PriceKey(105));

    asks.print_levels();

    println!();
    println!("--- BID SIDE (Descending) ---");
    let mut bids = BookSide::<Bids>::default();

    bids.level_mut(Reverse(PriceKey(105)));
    bids.level_mut(Reverse(PriceKey(101)));
    bids.level_mut(Reverse(PriceKey(103)));
    bids.level_mut(Reverse(PriceKey(99)));

    bids.print_levels();
}
