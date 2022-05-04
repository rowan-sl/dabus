use std::fmt::Debug;

use async_trait::async_trait;

use dabus::{
    event::EventType,
    stop::{EventActionType, EventArgs},
    BusInterface, BusStop, DABus,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut bus = DABus::new();
    bus.register(HelloHandler {});
    bus.register(Printer {});
    bus.register(Printer2 {});
    for _ in 0..10 {
        bus.query::<HelloHandler>(HelloMessage, "Hello, World!".to_string())
            .await
            .unwrap();
    }
}

#[derive(Debug, Clone)]
struct PrintMessage2;
#[derive(Debug)]
struct Printer2 {}

#[async_trait]
impl BusStop for Printer2 {
    type Event = PrintMessage2;
    type Args = String;
    type Response = ();

    /// handle a query-type event
    async fn query_event<'a>(
        &mut self,
        _args: EventArgs<'a, Self::Args>,
        _bus: BusInterface,
    ) -> Self::Response {
    }

    /// handle a send-type event
    async fn send_event<'a>(&mut self, args: EventArgs<'a, Self::Args>, _bus: BusInterface) {
        match args {
            EventArgs::HandleRef(to_print) => {
                println!("{}", to_print);
            }
            _ => {}
        }
    }

    /// after a type match check, how should this event be handled
    fn action(&mut self, _event: Self::Event, etype: EventType) -> EventActionType {
        match etype {
            EventType::Query => EventActionType::Ignore,
            EventType::Send => EventActionType::HandleRef,
        }
    }
}

#[derive(Debug, Clone)]
struct PrintMessage;
#[derive(Debug)]
struct Printer {}

#[async_trait]
impl BusStop for Printer {
    type Event = PrintMessage;
    type Args = Box<dyn Debug + Sync + Send>;
    type Response = String;

    /// handle a query-type event
    async fn query_event<'a>(
        &mut self,
        args: EventArgs<'a, Self::Args>,
        _bus: BusInterface,
    ) -> Self::Response {
        if let EventArgs::Consume(args) = args {
            format!("{:?}", args)
        } else {
            panic!()
        }
    }

    /// handle a send-type event
    async fn send_event<'a>(&mut self, _args: EventArgs<'a, Self::Args>, _bus: BusInterface) {}

    /// after a type match check, how should this event be handled
    fn action(&mut self, _event: Self::Event, etype: EventType) -> EventActionType {
        match etype {
            EventType::Query => EventActionType::Consume,
            EventType::Send => EventActionType::Ignore,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HelloMessage;
#[derive(Debug)]
pub struct HelloHandler {}

#[async_trait]
impl BusStop for HelloHandler {
    type Event = HelloMessage;
    type Args = String;
    type Response = ();

    /// handle a query-type event
    async fn query_event<'a>(
        &mut self,
        args: EventArgs<'a, Self::Args>,
        mut bus: BusInterface,
    ) -> Self::Response {
        if let EventArgs::Consume(args) = args {
            let to_print = bus.query::<Printer>(PrintMessage, Box::new(args)).await;
            bus.send::<Printer2>(PrintMessage2, to_print).await;
        } else {
            panic!()
        }
    }

    /// handle a send-type event
    async fn send_event<'a>(&mut self, _args: EventArgs<'a, Self::Args>, _bus: BusInterface) {}

    /// after a type match check, how should this event be handled
    fn action(&mut self, _event: Self::Event, etype: EventType) -> EventActionType {
        match etype {
            EventType::Query => EventActionType::Consume,
            EventType::Send => EventActionType::Ignore,
        }
    }
}
