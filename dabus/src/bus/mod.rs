pub mod sys;
mod async_util;

use std::any::TypeId;
use std::cell::RefCell;

use flume::{Receiver, Sender};
use uuid::Uuid;

use crate::event::BusEvent;
use crate::interface::BusInterface;
use crate::stop::{BusStop, BusStopMech};
use async_util::{OneOf, OneOfResult};


#[derive(Debug)]
pub struct DABus {
    global_event_recv: Receiver<(BusEvent, Sender<BusEvent>)>,
    global_event_send: Sender<(BusEvent, Sender<BusEvent>)>,
    registered_stops: RefCell<Vec<(Box<dyn BusStopMech>, TypeId)>>,
}

impl DABus {
    pub fn new() -> Self {
        let (global_event_send, global_event_recv): (_, Receiver<(BusEvent, Sender<BusEvent>)>) =
            flume::unbounded();
        Self {
            global_event_recv,
            global_event_send,
            registered_stops: RefCell::new(vec![]),
        }
    }

    /// Registers a new stop with the bus.
    pub fn register<B: BusStop + Send>(&mut self, stop: B) {
        self.registered_stops.borrow_mut().push((Box::new(stop), TypeId::of::<B>()));
    }

    // TODO implement this function once https://github.com/rust-lang/rust/issues/65991 is complete
    // pub fn deregister<B: BusStop + Send>(&mut self) -> Option<B> {
    //     self.registered_stops.borrow_mut().drain_filter(|stop| {
    //         stop.1 == TypeId::of::<B>()
    //     }).nth(0).map(|item| {*(item.0 as Box<dyn std::any::Any>).downcast().unwrap()})
    // }

    fn find_handler_for(&self, event: &BusEvent) -> Option<(Box<dyn BusStopMech>, TypeId)> {
        let mut who_asked = self
            .registered_stops
            .borrow_mut()
            .drain_filter(|stop| {
                // debug!("checking weather handler matches event: {:?}", stop);
                let matches = stop.0.cares(&*event);
                // debug!("Handler matches event: {}", matches);
                matches
            })
            .collect::<Vec<_>>();
        match who_asked.len() {
            0 => None,
            1 => who_asked.pop(),
            _ => {
                panic!("More than one handler asked!!!!");
            }
        }
    }

    #[async_recursion::async_recursion(?Send)]
    async fn fire_raw(
        &mut self,
        handler: &mut Box<dyn BusStopMech>,
        mut event: BusEvent,
    ) -> BusEvent {
        let id = event.uuid();

        let interface = BusInterface::new(self.global_event_send.clone());
        let mut stop_fut_container = Some(handler.raw_event(&mut event, interface));
        loop {
            let stop_fut = stop_fut_container.take().unwrap();
            let receiver = self.global_event_recv.clone();
            let recv_fut = receiver.recv_async();
            let bolth_fut = OneOf::new(stop_fut, recv_fut);
            match bolth_fut.await {
                OneOfResult::F0(stop_result, recv_fut) => {
                    // this means that the process is complete, and the result is done

                    drop(recv_fut); // we dont need this, nothing will be lost

                    assert!(stop_result.event_is::<sys::ReturnEvent>());
                    assert!(stop_result.uuid() == id);
                    return stop_result;
                }
                OneOfResult::F1(stop_fut, recv_result) => {
                    let recvd = recv_result.unwrap();
                    let mut handler = self.find_handler_for(&recvd.0).unwrap();
                    recvd
                        .1
                        .send(self.fire_raw(&mut handler.0, recvd.0).await)
                        .unwrap();
                    stop_fut_container = Some(stop_fut);
                    continue;
                }
                OneOfResult::All(_stop_result, _recv_result) => {
                    unreachable!(); // probably
                }
            };
        }
    }

    pub async fn fire<S: BusStop>(&mut self, event: S::Event, args: S::Args) -> S::Response {
        let id = Uuid::new_v4();
        let event = BusEvent::new(event, args, id);
        // info!("type of fired event: {} {} {}", type_name::<E>(), type_name::<A>(), type_name::<R>());
        // debug!("checking for handler for the new message");
        let mut handler = self
            .find_handler_for(&event)
            .expect("no handler for this message type exists");

        // look at this *very* clean code
        let res = *self
            .fire_raw(&mut handler.0, event)
            .await
            .into_raw()
            .unwrap()
            .1
            .downcast()
            .unwrap();

        self.registered_stops.borrow_mut().push(handler);

        res
    }
}

