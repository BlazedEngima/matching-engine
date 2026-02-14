#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct PriceKey(pub u64);

impl From<u64> for PriceKey {
    fn from(price: u64) -> Self {
        Self(price)
    }
}
