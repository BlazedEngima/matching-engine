use crate::orderbook::util::price_key::PriceKey;
use std::cmp::Reverse;

pub trait Side {
    type Key: Ord + Clone;

    fn key_to_price(key: Self::Key) -> PriceKey;

    fn compare_price(best: &Self::Key, limit: &Self::Key) -> bool {
        best > limit
    }
}

pub struct Bids;
impl Side for Bids {
    type Key = Reverse<PriceKey>;

    #[inline]
    fn key_to_price(key: Self::Key) -> PriceKey {
        key.0
    }
}

pub struct Asks;
impl Side for Asks {
    type Key = PriceKey;

    #[inline]
    fn key_to_price(key: Self::Key) -> PriceKey {
        key
    }
}
