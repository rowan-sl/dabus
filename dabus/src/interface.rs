use std::any::Any;

use flume::Sender;
use uuid::Uuid;

use crate::{event::BusEvent, bus::sys::ReturnEvent};


#[derive(Debug, Clone)]
pub struct BusInterface {
    event_queue: Sender<(BusEvent, flume::Sender<BusEvent>)>,
}

impl BusInterface {
    pub(crate) fn new(sender: Sender<(BusEvent, flume::Sender<BusEvent>)>) -> Self {
        Self {
            event_queue: sender,
        }
    }

    pub async fn fire<E: Any + 'static, A: Any + 'static, R: Any + 'static>(&mut self, event: E, args: A) -> R {
        // unbounded
        debug_assert!(self.event_queue.capacity().is_none());

        let (response_tx, response_rx) = flume::bounded(1);

        let id = Uuid::new_v4();
        let msg = BusEvent::new(event, args, id);

        self.event_queue
            .send((msg, response_tx))
            .expect("BusStops must be destroyed before the central handler!");

        *response_rx.recv_async().await.expect("sender sent a response").is_into::<ReturnEvent, R>().unwrap().1
    }
}
