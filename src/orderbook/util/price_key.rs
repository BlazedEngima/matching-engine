use std::fmt;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct PriceKey(pub u64);

impl fmt::Display for PriceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for PriceKey {
    fn from(price: u64) -> Self {
        Self(price)
    }
}
