use matching_engine::{data::fill_type::BookEvent, engine::matching_engine::Engine};
use rtrb::RingBuffer;
use std::thread;

fn main() {
    // Initialize SPSC ring_buffer to communicate to logging thread
    let (mut prod, mut cons) = RingBuffer::<BookEvent>::new(1 << 16);
    let engine = Engine::new(1 << 16);

    // Logging thread
    thread::spawn(move || {
        while let Ok(event) = cons.pop() {
            log_event(event);
        }
    });

    // Matching engine thread
    prod.push(event).ok(); // ideally avoid blocking
}
