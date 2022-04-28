#![feature(drain_filter)]

pub mod bus;
pub mod event;
pub mod interface;
pub mod prelude;
pub mod stop;

#[macro_use]
extern crate log;

pub use bus::DABus;
pub use event::BusEvent;
pub use interface::BusInterface;
pub use stop::BusStop;


#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    println!("Hello, world!");
}
