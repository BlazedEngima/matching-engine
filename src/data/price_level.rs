// Price is implicit from idx (Use price as index)
#[derive(Debug, Default, Eq, PartialEq, Hash)]
pub struct PriceLevel {
    pub head: Option<usize>,
    pub tail: Option<usize>,
    pub total_orders: u64,
}
