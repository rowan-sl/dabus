#![allow(clippy::missing_errors_doc)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[macro_use]
extern crate dabus;

use std::io::Write;

use anyhow::Result;

use dabus::{BusErrorUtil as _, BusInterface, BusStop, DABus, EventRegister};

async fn asmain() -> Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
    let mut bus = DABus::new();
    bus.register(STDOut);
    bus.register(Printer::new());
    bus.register(HelloHandler);
    match bus.fire(HELLO_EVENT, ()).await {
        Ok((_, trace)) => {
            info!("raw:\n{:#?}", trace);
            info!("formatted:\n{}", trace.display());
        }
        Err(trace) => {
            info!("error");
            info!("raw:\n{:#?}", trace);
            info!("formatted:\n{}", trace.display());
            info!("source:\n{:#?}", trace.source().unwrap().display());
        }
    }
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

event!(HELLO_EVENT, (), ());

#[derive(Debug)]
pub struct HelloHandler;

impl HelloHandler {
    async fn hello_world(&mut self, _: (), mut i: BusInterface) {
        i.fire(PRINT_EVENT, "Hello, World!".to_string())
            .await
            .unwrap_or_fwd(&i)
            .await;
        i.fire(FLUSH_EVENT, ()).await.unwrap_or_fwd(&i).await;
    }
}

impl BusStop for HelloHandler {
    fn registered_handlers(h: EventRegister<Self>) -> EventRegister<Self> {
        h.handler(HELLO_EVENT, Self::hello_world)
    }
}

event!(WRITE_EVENT, String, std::io::Result<()>);

#[derive(Debug)]
pub struct STDOut;

impl STDOut {
    pub async fn write(&mut self, data: String, _i: BusInterface) -> std::io::Result<()> {
        std::io::stdout().lock().write_all(data.as_bytes())
    }
}

impl BusStop for STDOut {
    fn registered_handlers(h: EventRegister<Self>) -> EventRegister<Self> {
        h.handler(WRITE_EVENT, Self::write)
    }
}

event!(PRINT_EVENT, String, ());
event!(FLUSH_EVENT, (), ());

#[derive(Debug)]
pub struct Printer {
    buffer: String,
}

impl Printer {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    async fn print(&mut self, to_print: String, _i: BusInterface) {
        self.buffer = format!("{}\n{}", self.buffer, to_print);
    }

    async fn flush(&mut self, _: (), mut i: BusInterface) {
        self.buffer.push('\n');
        i.fire(WRITE_EVENT, self.buffer.clone())
            .await
            .unwrap_or_fwd(&i)
            .await
            .unwrap();
        self.buffer.clear();
    }
}

impl BusStop for Printer {
    fn registered_handlers(h: EventRegister<Self>) -> EventRegister<Self> {
        h.handler(PRINT_EVENT, Self::print)
            .handler(FLUSH_EVENT, Self::flush)
    }
}
