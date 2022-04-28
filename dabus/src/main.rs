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

pub use bus::{DABus, sys::ReturnEvent};
pub use event::BusEvent;
pub use interface::BusInterface;
pub use stop::BusStop;


#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut bus = DABus::new();
    bus.register(HelloHandler {});
    bus.fire::<(), (), ()>((),()).await;
}

#[derive(Debug)]
struct HelloMessage {}

#[derive(Debug)]
struct HelloHandler {}

#[async_trait]
impl BusStop for HelloHandler {
    async fn event(
        &mut self,
        event: &mut BusEvent,
        _bus: BusInterface,
    ) -> BusEvent {
        let (_, _) = event.is_into::<(), ()>().unwrap();
        println!("Hello, World!");
        BusEvent::new(ReturnEvent, (), event.uuid())
    }

    fn cares(&mut self, event: &BusEvent) -> bool {
        event.args_are::<()>() &
        event.event_is::<()>()
    }
}
