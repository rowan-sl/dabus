#[allow(unused_imports)]
#[macro_use]
extern crate log;

use dabus2::{bus::DABus, stop::BusStop, EventDef, event::EventRegister};

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
    let mut bus = DABus::new();
    bus.register(Printer::new());
    bus.fire(PRINT_EVENT, "Hello, World!".to_string()).await;
    bus.fire(FLUSH_EVENT, ()).await;
}

static PRINT_EVENT: &'static EventDef<unique_type::new!(), String> = &unsafe { EventDef::new() };
static FLUSH_EVENT: &'static EventDef<unique_type::new!(), ()> = &unsafe { EventDef::new() };

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

    async fn print(&mut self, to_print: String) {
        self.buffer = format!("{}\n{}", self.buffer, to_print);
    }

    async fn flush(&mut self, _: ()) {
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
