use crate::data::book_event::BookEvent;
use std::fs::File;
use std::io::{BufWriter, Write};

pub struct BookLogger {
    writer: BufWriter<File>,
}

impl BookLogger {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    pub fn log(&mut self, event: &BookEvent) -> std::io::Result<()> {
        let line = match event {
            BookEvent::Match(event) => {
                format!(
                    "MATCH,maker({}),taker({}),price({}),qty({}),ts({})\n",
                    event.maker, event.taker, event.price, event.qty, event.ts
                )
            }
            BookEvent::Cancel(event) => {
                format!(
                    "CANCEL,id({}),qty({}),ts({})\n",
                    event.order_id, event.qty, event.ts
                )
            }
            BookEvent::Insert(event) => {
                format!(
                    "INSERT,id({}),price({}),qty({}),ts({})\n",
                    event.order_id, event.price, event.qty, event.ts
                )
            }
        };

        self.writer.write_all(line.as_bytes())
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
