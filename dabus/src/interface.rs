use crate::event::BusEvent;
use flume::Sender;

#[derive(Debug, Clone)]
pub struct BusInterface {
    event_queue: Sender<BusEvent>,
}

impl BusInterface {
    pub(crate) fn new(sender: Sender<BusEvent>) -> Self {
        Self {
            event_queue: sender,
        }
    }

    pub fn fire(&mut self, event: BusEvent) {
        // unbounded
        debug_assert!(self.event_queue.capacity().is_none());
        self.event_queue
            .send(event)
            .expect("BusStops must be destroyed before the central handler!");
    }
}
