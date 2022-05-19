use std::fmt::{Debug, Display};

use async_trait::async_trait;
use dabus::{
    decl_event,
    event::{BusEvent, EventType},
    stop::EventActionType,
    util::GeneralRequirements,
    BusInterface, BusStop, DABus,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut bus = DABus::new();
    bus.register(Printer);
    bus.register(Hello);
    bus.fire(HELLO_WORLD, ()).await.unwrap();
    let handler = bus.deregister::<Printer>();
    assert!(handler.is_some());
}

pub enum PrinterEvent {
    Display(Box<dyn Display + Sync + Send>),
    Debug((Box<dyn Debug + Sync + Send>, bool /* pretty-print */)),
    Print(String),
}

decl_event!(pub(self), PRINTER_DISPLAY, PrinterEvent, Display, Box<dyn Display + Sync + Send>,       String, None,     EventType::Query);
decl_event!(pub(self), PRINTER_DEBUG,   PrinterEvent, Debug,   (Box<dyn Debug + Sync + Send>, bool), String, None,     EventType::Query);
decl_event!(pub(self), PRINTER_PRINT,   PrinterEvent, Print,   String,                               (),     Some(()), EventType::Send);

#[derive(Debug)]
struct Printer;

#[async_trait]
impl BusStop for Printer {
    type Event = PrinterEvent;

    async fn event(
        &mut self,
        event: Self::Event,
        _bus: BusInterface,
    ) -> Option<Box<dyn GeneralRequirements + Send + 'static>> {
        match event {
            PrinterEvent::Debug((debuggable, prettyprint)) => {
                Some(Box::new(if prettyprint {
                    format!("{:#?}", debuggable)
                } else {
                    format!("{:?}", debuggable)
                }))
            }
            PrinterEvent::Display(displayable) => {
                Some(Box::new(format!("{}", displayable)))
            }
            PrinterEvent::Print(to_print) => {
                println!("{}", to_print);
                None
            }
        }
    }

    fn map_shared_event(
        &self,
        event: &BusEvent,
    ) -> Option<(Box<dyn FnOnce(BusEvent) -> Self::Event>, EventActionType)> {
        Some((Box::new(event.map_fn_if::<Self::Event, Self::Event, _>(|x| x)?), EventActionType::Consume))
    }
}

pub enum HelloEvent {
    Hello(()),
}

decl_event!(pub(self), HELLO_WORLD, HelloEvent, Hello, (), (), Some(()), EventType::Send);

#[derive(Debug)]
struct Hello;

#[async_trait]
impl BusStop for Hello {
    type Event = HelloEvent;

    async fn event(
        &mut self,
        event: Self::Event,
        mut bus: BusInterface,
    ) -> Option<Box<dyn GeneralRequirements + Send + 'static>> {
        match event {
            HelloEvent::Hello(()) => {
                let to_print = bus
                    .fire(
                        PRINTER_DEBUG,
                        (Box::new("Hello, World!".to_string()), false),
                    )
                    .await
                    .unwrap();
                bus.fire(PRINTER_PRINT, to_print).await.unwrap();
                None
            }
        }
    }

    fn map_shared_event(
        &self,
        event: &BusEvent,
    ) -> Option<(Box<dyn FnOnce(BusEvent) -> Self::Event>, EventActionType)> {
        Some((Box::new(event.map_fn_if::<Self::Event, Self::Event, _>(|x| x)?), EventActionType::Consume))
    }
}
