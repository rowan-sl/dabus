use std::{fmt::{Debug, Display}, any::Any};

use async_trait::async_trait;
use dabus::{
    event::EventType,
    stop::{EventActionType, EventArgs},
    BusInterface, BusStop, DABus,
    decl_event,
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

    async fn event<'a>(
        &mut self,
        event: EventArgs<'a, Self::Event>,
        _etype: EventType,
        _bus: BusInterface,
    ) -> Option<Box<dyn Any + Send + 'static>> {
        match event {
            EventArgs::Consume(PrinterEvent::Debug((debuggable, prettyprint))) => {
                Some(Box::new(if prettyprint {
                    format!("{:#?}", debuggable)
                } else {
                    format!("{:?}", debuggable)
                }))
            }
            EventArgs::Consume(PrinterEvent::Display(displayable)) => {
                Some(Box::new(format!("{}", displayable)))
            }
            EventArgs::HandleRef(PrinterEvent::Print(to_print)) => {
                println!("{}", to_print);
                None
            }
            _ => unreachable!()
        }
    }

    /// after a type match check, how should this event be handled
    fn action(
        &mut self,
        event: &Self::Event,
    ) -> EventActionType {
        match event {
            PrinterEvent::Display(..) => EventActionType::Consume,
            PrinterEvent::Debug(..) => EventActionType::Consume,
            PrinterEvent::Print(..) => EventActionType::HandleRef,
        }
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

    async fn event<'a>(
        &mut self,
        event: EventArgs<'a, Self::Event>,
        _etype: EventType,
        mut bus: BusInterface,
    ) -> Option<Box<dyn Any + Send + 'static>> {
        match event {
            EventArgs::Consume(HelloEvent::Hello(())) => {
                let to_print = bus.fire(PRINTER_DEBUG, (Box::new("Hello, World!".to_string()), false)).await.unwrap();
                bus.fire(PRINTER_PRINT, to_print).await.unwrap();
                None
            }
            _ => unreachable!()
        }
    }

    /// after a type match check, how should this event be handled
    fn action(
        &mut self,
        event: &Self::Event,
    ) -> EventActionType {
        match event {
            HelloEvent::Hello(()) => EventActionType::Consume
        }
    }
}
