pub mod emergency;

use flume::{Receiver, Sender};

use crate::event::BusEvent;
use crate::interface::BusInterface;
use crate::stop::BusStop;

#[derive(Debug)]
pub struct DABus {
    global_event_recv: Receiver<BusEvent>,
    global_event_send: Sender<BusEvent>,
    registered_stops: Vec<Box<dyn BusStop>>,
}

impl DABus {
    pub fn new() -> Self {
        let (global_event_send, global_event_recv): (_, Receiver<BusEvent>) = flume::unbounded();
        Self {
            global_event_recv,
            global_event_send,
            registered_stops: vec![],
        }
    }

    /// Registers a new stop with the bus.
    pub fn register<B: BusStop>(&mut self, stop: B) {
        self.registered_stops.push(Box::new(stop));
    }

    pub fn manually_feed(&mut self, event: BusEvent) {
        self.global_event_send.send(event).unwrap();
    }

    pub fn run(&mut self) {
        'main: loop {
            let mut event = self.global_event_recv.recv().unwrap();

            if event.is_into::<emergency::Exit>().is_some() {
                break 'main;
            }

            'consumer: for stop in &mut self.registered_stops {
                stop.event(
                    &mut event,
                    BusInterface::new(self.global_event_send.clone()),
                );
                if event.consumed() {
                    break 'consumer;
                }
            }

            if !event.consumed() {
                warn!("Unhandled bus event");
            }
        }
    }
}
