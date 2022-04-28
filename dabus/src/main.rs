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

pub use bus::{sys::ReturnEvent, DABus};
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
    bus.register(Printer {});
    bus.fire::<HelloMessage, (String,), ()>(HelloMessage, ("Hello, World!".to_string(),))
        .await;
}

#[derive(Debug)]
struct PrintMessage;
#[derive(Debug)]
struct Printer {}

#[async_trait]
impl BusStop for Printer {
    async fn event(&mut self, event: &mut BusEvent, _bus: BusInterface) -> BusEvent {
        let (_, args) = event
            .is_into::<PrintMessage, (Box<dyn Debug + Send>,)>()
            .unwrap();
        let res = format!("{:#?}", args.0);
        BusEvent::new(ReturnEvent, res, event.uuid())
    }

    fn cares(&mut self, event: &BusEvent) -> bool {
        type Event = PrintMessage;
        type Args = (Box<dyn Debug + Send>,);
        event.event_is::<Event>() & event.args_are::<Args>()
    }
}

#[derive(Debug)]
struct HelloMessage;

#[derive(Debug)]
struct HelloHandler {}

#[async_trait]
impl BusStop for HelloHandler {
    async fn event(&mut self, event: &mut BusEvent, mut bus: BusInterface) -> BusEvent {
        let (_, args) = event.is_into::<HelloMessage, (String,)>().unwrap();
        println!(
            "{}",
            bus.fire::<_, (Box<dyn Debug + Send>,), String>(PrintMessage, (Box::new(args.0),))
                .await
        );
        BusEvent::new(ReturnEvent, (), event.uuid())
    }

    fn cares(&mut self, event: &BusEvent) -> bool {
        type Event = HelloMessage;
        type Args = (String,);
        event.event_is::<Event>() & event.args_are::<Args>()
    }
}
