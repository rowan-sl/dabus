mod async_util;
pub mod sys;

use std::any::TypeId;
use std::cell::RefCell;

use flume::{Receiver, Sender};
use futures::FutureExt;
use uuid::Uuid;

use crate::event::{BusEvent, EventType};
use crate::interface::{BusInterface, RequestType};
use crate::stop::{BusStop, BusStopMech, EventActionType, RawEventReturn};
use async_util::{OneOf, OneOfResult};

#[derive(Debug)]
pub struct DABus {
    global_event_recv: Receiver<(BusEvent, RequestType)>,
    global_event_send: Sender<(BusEvent, RequestType)>,
    registered_stops: RefCell<Vec<(Box<dyn BusStopMech>, TypeId)>>,
}

impl DABus {
    pub fn new() -> Self {
        let (global_event_send, global_event_recv): (_, Receiver<(BusEvent, RequestType)>) =
            flume::unbounded();
        Self {
            global_event_recv,
            global_event_send,
            registered_stops: RefCell::new(vec![]),
        }
    }

    /// Registers a new stop with the bus.
    pub fn register<B: BusStop + Send>(&mut self, stop: B) {
        self.registered_stops
            .borrow_mut()
            .push((Box::new(stop), TypeId::of::<B>()));
    }

    // TODO implement this function once https://github.com/rust-lang/rust/issues/65991 is complete
    // pub fn deregister<B: BusStop + Send>(&mut self) -> Option<B> {
    //     self.registered_stops.borrow_mut().drain_filter(|stop| {
    //         stop.1 == TypeId::of::<B>()
    //     }).nth(0).map(|item| {*(item.0 as Box<dyn std::any::Any>).downcast().unwrap()})
    // }

    fn get_handlers(
        &mut self,
        event: &BusEvent,
        etype: EventType,
    ) -> Result<Vec<(Box<dyn BusStopMech>, TypeId, EventActionType)>, GetHandlersError> {
        let mut handlers = self
            .registered_stops
            .borrow_mut()
            .drain_filter(|stop| {
                if stop.0.matches(event) {
                    let action = stop.0.raw_action(event, etype);
                    EventActionType::Ignore != action
                } else {
                    false
                }
            })
            .map(|(mut stop, stop_id)| {
                let action = stop.raw_action(event, etype);
                (stop, stop_id, action)
            })
            .collect::<Vec<_>>();
        if handlers.is_empty() {
            Err(GetHandlersError::NoHandler)
        } else {
            handlers.sort_by(|a, b| {
                use std::cmp::Ordering::{Equal, Greater, Less};
                use EventActionType::{Consume, HandleCopy, HandleRef, Ignore};
                match (a.2, b.2) {
                    (Ignore, Ignore) => Equal,
                    (Ignore, _) => Less,
                    (_, Ignore) => Greater,
                    (Consume, Consume) => Equal,
                    (_, Consume) => Less,
                    (Consume, _) => Greater,
                    (HandleCopy, HandleRef) => Equal,
                    (HandleRef, HandleCopy) => Equal,
                    (HandleCopy, HandleCopy) => Equal,
                    (HandleRef, HandleRef) => Equal,
                }
            });
            if handlers.iter().fold(0usize, |mut acc, elem| {
                if elem.2 == EventActionType::Consume {
                    acc += 1;
                }
                acc
            }) > 1
            {
                self.registered_stops
                    .borrow_mut()
                    .extend(&mut handlers.into_iter().map(|(a, b, _)| (a, b)));
                Err(GetHandlersError::MultipleConsume)
            } else {
                match etype {
                    EventType::Query => {
                        if handlers.len() > 1 {
                            self.registered_stops
                                .borrow_mut()
                                .extend(&mut handlers.into_iter().map(|(a, b, _)| (a, b)));
                            Err(GetHandlersError::MultipleQuery)
                        } else {
                            Ok(handlers)
                        }
                    }
                    EventType::Send => Ok(handlers),
                }
            }
        }
    }

    #[async_recursion::async_recursion(?Send)]
    async fn query_raw(&mut self, raw_event: BusEvent) -> Result<BusEvent, FireEventError> {
        let mut handler = self.get_handlers(&raw_event, EventType::Query)?.remove(0);

        let id = raw_event.uuid();
        // not really needed here, mostly for supporting send events. holds on to the original BusEvent
        let mut event_container = Some(raw_event);

        let mut stop_fut_container = Some(handler.0.raw_event(
            &mut event_container,
            EventType::Query,
            BusInterface::new(self.global_event_send.clone()),
        ));

        let response = 'poll: loop {
            match OneOf::new(
                stop_fut_container.take().unwrap(),
                self.global_event_recv.clone().into_recv_async(),
            ).await {
                OneOfResult::F0(stop_result, ..) => {
                    // this means that the process is complete, and the result is done

                    match stop_result {
                        RawEventReturn::Response(response) => {
                            debug_assert!(response.event_is::<sys::ReturnEvent>());
                            debug_assert_eq!(response.uuid(), id);
                            break 'poll response;
                        }
                        _ => unreachable!(),
                    };
                }
                OneOfResult::F1(stop_fut, recv_result) => {
                    let recvd = recv_result.unwrap();
                    match recvd.1 {
                        RequestType::Query { responder } => {
                            responder.send(self.query_raw(recvd.0).await?).unwrap();
                            stop_fut_container = Some(stop_fut)
                        }
                        RequestType::Send { notifier } => {
                            self.send_raw(recvd.0).await?;
                            notifier.send(()).unwrap();
                            stop_fut_container = Some(stop_fut)
                        }
                    }
                }
                OneOfResult::All(..) => unreachable!()
            };
        };
        drop(stop_fut_container); //to please the gods
        self.registered_stops
            .borrow_mut()
            .push((handler.0, handler.1));
        Ok(response)
    }

    pub async fn query<S: BusStop>(
        &mut self,
        event: S::Event,
        args: S::Args,
    ) -> Result<S::Response, FireEventError> {
        let id = Uuid::new_v4();
        let event = BusEvent::new(event, args, id);

        // look at this *very* clean code
        let res: S::Response = match self.query_raw(event).await?.into_raw().1.downcast() {
            Ok(expected) => *expected,
            Err(..) => {
                warn!("Mismatched return types are allways dropped, this could cause issues");
                return Err(FireEventError::InvalidReturnType);
            }
        };

        Ok(res)
    }

    #[async_recursion::async_recursion(?Send)]
    async fn send_raw(&mut self, raw_event: BusEvent) -> Result<(), FireEventError> {
        let mut handler_ids = vec![];
        for (handler, id, method) in self.get_handlers(&raw_event, EventType::Send)? {
            self.registered_stops.borrow_mut().push((handler, id));
            handler_ids.push((id, method));
        }

        let mut event_container = Some(raw_event);

        for (handler_id, _) in handler_ids {
            let mut handler = self
                .registered_stops
                .borrow_mut()
                .drain_filter(|stop| stop.1 == handler_id)
                .nth(0)
                .unwrap();

            let mut stop_fut_container = Some(handler.0.raw_event(
                &mut event_container,
                EventType::Send,
                BusInterface::new(self.global_event_send.clone()),
            ));
            'poll: loop {
                match OneOf::new(
                    stop_fut_container.take().unwrap(),
                    self.global_event_recv.clone().into_recv_async(),
                ).await {
                    OneOfResult::F0(..) => break 'poll,
                    OneOfResult::F1(stop_fut, recv_result) => {
                        let (event, rtype) = recv_result.unwrap();
                        match rtype {
                            RequestType::Query { responder } => {
                                responder.send(self.query_raw(event).await?).unwrap();
                                stop_fut_container = Some(stop_fut)
                            }
                            RequestType::Send { notifier } => {
                                self.send_raw(event).await?;
                                notifier.send(()).unwrap();
                                stop_fut_container = Some(stop_fut)
                            }
                        }
                    }
                    OneOfResult::All(..) => unreachable!()
                };
            }
            drop(stop_fut_container); //to please the gods
            self.registered_stops
                .borrow_mut()
                .push((handler.0, handler.1));
        }

        Ok(())
    }

    pub async fn send<S: BusStop>(
        &mut self,
        event: S::Event,
        args: S::Args,
    ) -> Result<(), FireEventError> {
        let id = Uuid::new_v4();
        let event = BusEvent::new(event, args, id);

        self.send_raw(event).await
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum FireEventError {
    #[error("Could not find an appropreate handler for this event: {0}")]
    Handler(#[from] GetHandlersError),
    /// note: this will be phased out in the future, once handler selection relies on the handler type
    #[error("Handler did not return the specified return type!")]
    InvalidReturnType,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum GetHandlersError {
    #[error("Multiple consume level handlers!")]
    MultipleConsume,
    #[error("Multiple handlers responded in query mode!")]
    MultipleQuery,
    #[error("Could not find a handler for the event!")]
    NoHandler,
}
