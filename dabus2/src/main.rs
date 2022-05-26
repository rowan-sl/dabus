#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[macro_use]
extern crate dabus2;

use anyhow::Result;

use dabus2::{DABus, BusStop, EventRegister, BusInterface};

// #[tokio::main]
async fn asmain() -> Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
    let mut bus = DABus::new();
    bus.register(Printer::new());
    bus.fire(PRINT_EVENT, "Hello, World!".to_string()).await?;
    bus.fire(FLUSH_EVENT, ()).await?;
    Ok(())
}

#[cfg(not(miri))]
fn main() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(asmain())
}

// custom builder with no io support enabled so that it runs under miri
#[cfg(miri)]
fn main() -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?
        .block_on(asmain())
}

event!(PRINT_EVENT, String, ());
event!(FLUSH_EVENT, (), ());


#[derive(Debug)]
pub struct Printer {
    buffer: String,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    async fn print(&mut self, to_print: String, _i: BusInterface) {
        self.buffer = format!("{}\n{}", self.buffer, to_print);
    }

    async fn flush(&mut self, _: (), _i: BusInterface) {
        println!("{}", self.buffer);
    }
}

impl BusStop for Printer {
    fn registered_handlers(h: EventRegister<Self>) -> EventRegister<Self>
    {
        h.handler(PRINT_EVENT, Self::print)
            .handler(FLUSH_EVENT, Self::flush)
    }
}
