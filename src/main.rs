use clap::Parser;
use matching_engine::data::book_event::BookEvent;
use matching_engine::data::order_types::IncomingOrder;
use matching_engine::engine::matching_engine::Engine;
use matching_engine::input::generator::Generator;
use matching_engine::input::replay_reader::ReplayReader;
use matching_engine::logger::book_logger::BookLogger;
use rtrb::RingBuffer;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

#[derive(Parser, Debug)]
struct Args {
    /// gen or replay
    #[arg(long)]
    mode: String,

    #[arg(long, default_value = "replay_input.csv")]
    replay_output: String,

    #[arg(long, default_value = "12345")]
    seed: u64,

    #[arg(long, default_value = "10000")]
    mid_price: i64,

    #[arg(long, default_value = "10000")]
    num_of_events: usize,

    /// Input file for replay mode
    #[arg(long)]
    input: Option<String>,

    /// Output file
    #[arg(long, default_value = "output.log")]
    output: String,
}

const DEFAULT_SIZE: usize = 1 << 16;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Input generation of orders
    let input_events: Vec<IncomingOrder> = match args.mode.as_str() {
        "gen" => {
            println!("Generating random input...");
            let mut generator = Generator::new(args.seed, args.mid_price, &args.replay_output)?;
            generator.generate(args.num_of_events) // generate N events
        }
        "replay" => {
            println!("Loading replay file...");
            let input = args.input.expect("Replay mode requires --input <file>");
            let mut reader = ReplayReader::from_file(input.as_str())?;
            reader.parse_orders()
        }
        _ => panic!("Invalid mode. Use --mode gen or --mode replay"),
    };

    println!("Loaded {} input events", input_events.len());

    // Init ring buffer and syncing atmoic bool
    let mut engine = Engine::new(1 << 16);
    let (mut producer, mut consumer) = RingBuffer::<BookEvent>::new(DEFAULT_SIZE);
    let done = Arc::new(AtomicBool::new(false));
    let done_producer = done.clone();
    let output_path = args.output;

    // Spawn engine(producer) thread
    let engine_handle = thread::spawn(move || -> anyhow::Result<()> {
        // Perform main engine matching task
        for order in input_events {
            let events = engine.match_order(order);
            for event in events {
                producer.push(event)?;
            }
        }

        let book_state = engine.get_book().print_book();
        // Should only be one element
        for event in book_state {
            producer.push(event)?;
        }
        done_producer.store(true, Ordering::Release);
        Ok(())
    });

    let mut logger = BookLogger::new(&output_path)?;
    loop {
        match consumer.pop() {
            Ok(event) => {
                logger.log(&event)?;
            }
            Err(_) => {
                if done.load(Ordering::Acquire) {
                    break;
                }
                // Otherwise just continue trying
            }
        }
    }

    logger.flush()?;
    engine_handle.join().unwrap()?;

    println!("Done.");

    Ok(())
}
