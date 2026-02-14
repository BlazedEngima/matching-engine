use crate::data::order_types::IncomingOrder;
use crate::input::traits::EventSource;
use std::io::BufRead;

pub struct ReplayFile<R: BufRead> {
    reader: R,
}

impl<R: BufRead> EventSource for ReplayFile<R> {
    fn next_event(&mut self) -> Option<IncomingOrder> {
        let mut line = String::new();
        if self.reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        parse_event(&line)
    }
}
