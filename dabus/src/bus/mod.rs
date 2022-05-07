mod async_util;
pub mod sys;

use std::any::TypeId;
use std::cell::RefCell;

use flume::{Receiver, Sender};
use uuid::Uuid;

use crate::event::{BusEvent, EventType};
use crate::interface::{BusInterface, InterfaceEvent};
use crate::stop::{BusStop, BusStopMech, EventActionType, RawEventReturn};
use crate::args::EventSpec;
use crate::util::{GeneralRequirements, PossiblyClone};
use async_util::{OneOf, OneOfResult};

#[derive(Debug)]
pub struct DABus {
    global_event_recv: Receiver<InterfaceEvent>,
    global_event_send: Sender<InterfaceEvent>,
    registered_stops: RefCell<Vec<(Box<dyn BusStopMech + Send + Sync + 'static>, TypeId)>>,
}

impl DABus {
    pub fn new() -> Self {
        let (global_event_send, global_event_recv): (_, Receiver<InterfaceEvent>) =
            flume::unbounded();
        Self {
            global_event_recv,
            global_event_send,
            registered_stops: RefCell::new(vec![]),
        }
    }

    /// Registers a new stop with the bus.
    pub fn register<B: BusStop + Send + Sync + 'static>(&mut self, stop: B) {
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
    ) -> Result<
        Vec<(
            Box<dyn BusStopMech + Send + Sync + 'static>,
            TypeId,
            EventActionType,
        )>,
        GetHandlersError,
    > {
        let mut handlers = self
            .registered_stops
            .borrow_mut()
            .drain_filter(|stop| {
                trace!("Checking stop {:#?}", stop);
                let matches = stop.0.matches(event);
                trace!("Stop matches event: {}", matches);
                if matches {
                    let action = stop.0.raw_action(event);
                    EventActionType::Ignore != action
                } else {
                    false
                }
            })
            .map(|(mut stop, stop_id)| {
                let action = stop.raw_action(event);
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

    #[async_recursion::async_recursion]
    async fn handle_event_inner(
        &mut self,
        event_container: &mut Option<BusEvent>,
        mut handler: (Box<dyn BusStopMech + Send + Sync + 'static>, TypeId),
        etype: EventType,
    ) -> Result<Option<BusEvent>, FireEventError> {
        let id = event_container.as_ref().unwrap().uuid();

        let mut stop_fut_container = Some(handler.0.raw_event(
            event_container,
            etype,
            BusInterface::new(self.global_event_send.clone()),
        ));

        let response = 'poll: loop {
            match OneOf::new(
                stop_fut_container.take().unwrap(),
                self.global_event_recv.clone().into_recv_async(),
            )
            .await
            {
                OneOfResult::F0(stop_result, ..) => {
                    // this means that the process is complete, and the result is done

                    match stop_result {
                        RawEventReturn::Response(response) => {
                            debug_assert_eq!(response.uuid(), id);
                            break 'poll Some(response);
                        }
                        RawEventReturn::Processed => break 'poll None,
                        RawEventReturn::Ignored => unreachable!(),
                    };
                }
                OneOfResult::F1(stop_fut, recv_result) => {
                    match recv_result.unwrap() {
                        InterfaceEvent::Call(event, etype, responeder) => {
                            responeder.send(self.handle_event(event, etype).await).unwrap();
                            stop_fut_container = Some(stop_fut);
                        }
                        InterfaceEvent::FwdErr(error) => {
                            drop(stop_fut);
                            return Err(error);
                            //TODO add more logic for backtraces
                        }
                    }
                }
                OneOfResult::All(..) => unreachable!(),
            };
        };
        drop(stop_fut_container); //to please the gods
        self.registered_stops
            .borrow_mut()
            .push((handler.0, handler.1));
        Ok(response)
    }

    async fn handle_event(
        &mut self,
        raw_event: BusEvent,
        etype: EventType,
    ) -> Result<Option<BusEvent>, FireEventError> {
        let mut handler_ids = vec![];
        for (handler, id, method) in self.get_handlers(&raw_event, etype)? {
            self.registered_stops.borrow_mut().push((handler, id));
            handler_ids.push((id, method));
        }

        let mut event_container = Some(raw_event);

        for (handler_id, _) in handler_ids {
            let handler = self
                .registered_stops
                .borrow_mut()
                .drain_filter(|stop| stop.1 == handler_id)
                .nth(0)
                .unwrap();

            match self
                .handle_event_inner(&mut event_container, handler, etype)
                .await?
            {
                Some(response) => {
                    // it must have been a query event, so there wont be any more reponses
                    return Ok(Some(response));
                }
                None => {}
            }
        }

        Ok(None)
    }

    pub async fn fire<S: Send + 'static, A: Send + Sync, R: GeneralRequirements + Send + Sync + 'static>(&mut self, q: &'static EventSpec<S, A, R>, args: A) -> Result<R, FireEventError> {
        let etype = q.event_variant.clone();
        let args_as_sum_t = (q.convert)(args);

        let raw_event = BusEvent::new(args_as_sum_t, Uuid::new_v4());
        let response = self.handle_event(raw_event, etype).await?;
        match response {
            Some(res) => {
                match res.is_into::<R>() {
                    Ok(expected) => {
                        Ok(*expected)
                    }
                    Err(actual) => {
                        let expected = std::any::type_name::<Box<R>>();
                        let found = (*actual.into_raw().0).type_name();
                        Err(FireEventError::InvalidReturnType(expected, found))
                    }
                }
            }
            None => {
                Ok(q.default_return.as_ref().expect("Send type events must provide a default return").try_clone())
            }
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum FireEventError {
    #[error("Could not find an appropreate handler for this event: {0}")]
    Handler(#[from] GetHandlersError),
    /// note: this will be phased out in the future, once handler selection relies on the handler type
    ///
    /// (expected, found)
    #[error("Handler did not return the specified return type! expected {0}, found {1}")]
    InvalidReturnType(&'static str, &'static str),
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
