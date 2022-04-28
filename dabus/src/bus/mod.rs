pub mod emergency;
pub mod sys;

use std::{any::Any, pin::Pin, cell::RefCell};

use flume::{Receiver, Sender};
use uuid::Uuid;
use futures::{Future, task::{Poll, Context}, FutureExt};

use crate::event::BusEvent;
use crate::interface::BusInterface;
use crate::stop::BusStop;


#[derive(Debug)]
pub struct DABus {
    global_event_recv: Receiver<(BusEvent, Sender<BusEvent>)>,
    global_event_send: Sender<(BusEvent, Sender<BusEvent>)>,
    registered_stops: RefCell<Vec<Box<dyn BusStop>>>,
}

impl DABus {
    pub fn new() -> Self {
        let (global_event_send, global_event_recv): (_, Receiver<(BusEvent, Sender<BusEvent>)>) = flume::unbounded();
        Self {
            global_event_recv,
            global_event_send,
            registered_stops: RefCell::new(vec![]),
        }
    }

    /// Registers a new stop with the bus.
    pub fn register<B: BusStop>(&mut self, stop: B) {
        self.registered_stops.borrow_mut().push(Box::new(stop));
    }

    fn find_handler_for(&self, event: &BusEvent) -> Option<Box<dyn BusStop>> {
        let mut who_asked = self.registered_stops.borrow_mut().drain_filter(|stop| {
            stop.cares(event)
        }).collect::<Vec<_>>();
        match who_asked.len() {
            0 => None,
            1 => {
                who_asked.pop()
            }
            _ => {
                panic!("More than one handler asked!!!!");
            }
        }
    }

    #[async_recursion::async_recursion(?Send)]
    async fn fire_raw(&mut self, handler: &mut Box<dyn BusStop>, mut event: BusEvent) -> BusEvent {
        let id = event.uuid();

        let interface = BusInterface::new(self.global_event_send.clone());
        let mut stop_fut_container = Some(handler.event(&mut event, interface));
        loop {
            let stop_fut = stop_fut_container.take().unwrap();
            let receiver = self.global_event_recv.clone();
            let recv_fut = receiver.recv_async();
            let bolth_fut = OneOf::new(stop_fut, recv_fut);
            match bolth_fut.await {
                OneOfResult::F0(stop_result, recv_fut) => {
                    // this means that the process is complete, and the result is done

                    drop(recv_fut);// we dont need this, nothing will be lost

                    assert!(stop_result.event_is::<sys::ReturnEvent>());
                    assert!(stop_result.uuid() == id);
                    return stop_result;
                }
                OneOfResult::F1(stop_fut, recv_result) => {
                    let recvd = recv_result.unwrap();
                    let mut handler = self.find_handler_for(&recvd.0).unwrap();
                    recvd.1.send(self.fire_raw(&mut handler, recvd.0).await).unwrap();
                    stop_fut_container = Some(stop_fut);
                    continue;
                }
                OneOfResult::All(_stop_result, _recv_result) => {
                    unreachable!(); // probably
                }
            };
        };
    }

    pub async fn fire<E: Any + 'static, A: Any + 'static, R: Any + 'static>(&mut self, event: E, args: A) -> R {
        let id = Uuid::new_v4();
        let event = BusEvent::new(event, args, id);

        let mut handler = self.find_handler_for(&event).unwrap();

        let res = *self.fire_raw(&mut handler, event).await.into_raw().unwrap().1.downcast().unwrap();

        self.registered_stops.borrow_mut().push(handler);

        res
    }
}

struct OneOf<F0: Future, F1: Future>{
    fut0: Option<F0>,
    fut1: Option<F1>,
}

impl<F0: Future, F1: Future> OneOf<F0, F1> {
    pub fn new(f0: F0, f1: F1) -> Self {
        Self {
            fut0: Some(f0),
            fut1: Some(f1),
        }
    }
}

enum OneOfResult<F0: Future, F1: Future> {
    F0(F0::Output, F1),
    F1(F0, F1::Output),
    All(F0::Output, F1::Output),
}

impl<F0: Future + Unpin, F1: Future + Unpin> Future for OneOf<F0, F1> {
    type Output = OneOfResult<F0, F1>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match (self.fut0.as_mut().unwrap().poll_unpin(cx), self.fut1.as_mut().unwrap().poll_unpin(cx)) {
            (Poll::Pending, Poll::Pending) => {
                Poll::Pending
            }
            (Poll::Ready(f0), Poll::Pending) => {
                Poll::Ready(OneOfResult::F0(f0, self.fut1.take().unwrap()))
            }
            (Poll::Pending, Poll::Ready(f1)) => {
                Poll::Ready(OneOfResult::F1(self.fut0.take().unwrap(), f1))
            }
            (Poll::Ready(f0), Poll::Ready(f1)) => {
                Poll::Ready(OneOfResult::All(f0, f1))
            }
        }
    }
}
