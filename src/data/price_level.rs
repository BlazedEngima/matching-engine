#[derive(Debug, Default, Eq, PartialEq)]
// Price is implicit from idx (Use price as index)
pub struct PriceLevel {
    pub head: Option<usize>,
    pub tail: Option<usize>,
    pub total_orders: u64,
}
