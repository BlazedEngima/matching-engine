use crate::data::order_types::IncomingOrder;

pub trait EventSource {
    fn next_event(&mut self) -> Option<IncomingOrder>;
}
