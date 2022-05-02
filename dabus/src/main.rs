#![feature(drain_filter)]

pub mod bus;
pub mod event;
pub mod interface;
pub mod prelude;
pub mod stop;

#[macro_use]
extern crate log;
#[macro_use]
extern crate async_trait;

use std::fmt::Debug;

pub use bus::DABus;
pub use interface::BusInterface;
pub use stop::BusStop;


#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut bus = DABus::new();
    bus.register(HelloHandler {});
    bus.register(Printer {});
    bus.fire::<HelloHandler>(HelloMessage, "Hello, World!".to_string())
        .await
        .unwrap();
}

#[derive(Debug)]
struct PrintMessage;
#[derive(Debug)]
struct Printer {}

#[async_trait]
impl BusStop for Printer {
    type Event = PrintMessage;
    type Args = Box<dyn Debug + Send>;
    type Response = String;

    async fn event(
        &mut self,
        _event: Self::Event,
        args: Self::Args,
        _bus: BusInterface,
    ) -> Self::Response {
        format!("{:#?}", args)
    }
}

#[derive(Debug)]
struct HelloMessage;
#[derive(Debug)]
struct HelloHandler {}

#[async_trait]
impl BusStop for HelloHandler {
    type Event = HelloMessage;
    type Args = String;
    type Response = ();

    async fn event(
        &mut self,
        _event: Self::Event,
        args: Self::Args,
        mut bus: BusInterface,
    ) -> Self::Response {
        println!(
            "{}",
            bus.fire::<Printer>(PrintMessage, Box::new(args)).await
        );
    }
}
