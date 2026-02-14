use crate::input::traits::EventSource;

pub struct GeneratedInput {
    // config: rate, distribution, etc
}

impl EventSource for GeneratedInput {
    fn next_event(&mut self) -> Option<Event> {
        Some(generate_event())
    }
}
