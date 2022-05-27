pub mod error;

use core::any::TypeId;
use std::{fmt::Debug, sync::Arc};

use flume::{r#async::RecvFut, Receiver, Sender};
use futures::future::BoxFuture;

use crate::{
    core::dyn_var::DynVar,
    event::EventDef,
    interface::{BusInterface, BusInterfaceEvent},
    stop::{BusStopContainer, BusStopMechContainer},
    util::{
        async_util::{OneOf, OneOfResult},
        dyn_debug::DynDebug,
    },
    BusStop, EventRegister, bus::error::{CallTrace, CallEvent},
};
use error::{BaseFireEventError, FireEventError};

use self::error::Resolution;

enum Frame {
    ReadyToPoll {
        interface_recv: Receiver<BusInterfaceEvent>,
        recev_fut: RecvFut<'static, BusInterfaceEvent>,
        handler: Arc<BusStopContainer>,
        handler_fut: BoxFuture<'static, DynVar>,
    },
    AwaitingNestedCall {
        interface_recv: Receiver<BusInterfaceEvent>,
        handler: Arc<BusStopContainer>,
        handler_fut: BoxFuture<'static, DynVar>,
        responder: Sender<Result<DynVar, FireEventError>>,
    },
}

#[derive(Debug)]
pub struct DABus {
    registered_stops: Vec<BusStopContainer>,
}

impl DABus {
    pub const fn new() -> Self {
        Self {
            registered_stops: vec![],
        }
    }

    pub fn register<T: BusStop + Debug + Send + Sync + 'static>(&mut self, stop: T) {
        info!("Registering stop {:?}", stop);
        debug!(
            "Stop handlers: {:#?}",
            <T as BusStop>::registered_handlers(EventRegister::new())
                .handlers
                .into_iter()
                .map(|h| { h.2 })
                .collect::<Vec<_>>()
        );
        self.registered_stops
            .push(BusStopContainer::new(Box::new(BusStopMechContainer::new(
                stop,
            ))));
    }

    pub fn deregister<T: BusStop + Debug + Send + Sync + 'static>(&mut self) -> Option<T> {
        let stop = self
            .registered_stops
            .drain_filter(|stop| (*stop.inner.try_lock().unwrap()).as_any().type_id() == TypeId::of::<T>())
            .nth(0)
            .map(|item| *item.inner.into_inner().to_any().downcast().unwrap());
        info!("Deregistering stop {:?}", stop);
        stop
    }

    fn handlers_for(&mut self, def: TypeId) -> Vec<BusStopContainer> {
        debug!("Looking for handlers for {:?}", def);
        self.registered_stops
            .drain_filter(|stop| {
                if stop.relevant(def) {
                    trace!("Found match: {:?}", stop.debug());
                    true
                } else {
                    trace!("Mismatch: {:?}", stop.debug());
                    false
                }
            })
            .collect()
    }

    fn gen_frame_for(&mut self, def: TypeId, args: DynVar) -> Result<Frame, FireEventError> {
        let mut handlers = self.handlers_for(def);
        assert!(
            handlers.len() < 2,
            "currently only supports one handler for an event! this WILL change soonTM"
        );
        if handlers.is_empty() {
            error!("no handlers found for {:?}", def);
            Err(FireEventError::from(BaseFireEventError::NoHandler))?
        }
        let handler = handlers.remove(0);
        let (interface_send, interface_recv): (Sender<BusInterfaceEvent>, _) = flume::bounded(1);
        let interface = BusInterface::new(interface_send);

        let recev_fut = interface_recv.clone().into_recv_async();
        let shared_handler = Arc::new(handler);
        let handler_fut = unsafe { shared_handler.clone().handle_raw_event(def, args, interface) };

        let frame = Frame::ReadyToPoll {
            interface_recv,
            recev_fut,
            handler: shared_handler,
            handler_fut: Box::pin(async move { handler_fut.await }),
        };

        Ok(frame)
    }

    async fn raw_fire(&mut self, def: TypeId, args: DynVar, mut trace: CallTrace) -> (Option<DynVar>, CallTrace) {
        let mut stack: Vec<Frame> = vec![];

        stack.push(match self.gen_frame_for(def, args) {
            Ok(initial_frame) => initial_frame,
            Err(initial_frame_error) => {
                trace.root.resolution = Some(Resolution::BusError(initial_frame_error));
                return (None, trace)
            }
        });

        'main: loop {
            match stack.pop().unwrap() {
                Frame::ReadyToPoll {
                    interface_recv,
                    recev_fut,
                    handler,
                    handler_fut,
                } => {
                    let recv_and_handler_fut = OneOf::new(recev_fut, handler_fut);
                    match recv_and_handler_fut.await {
                        OneOfResult::F0(interface_event, handler_fut) => {
                            match interface_event.unwrap() {
                                BusInterfaceEvent::Fire {
                                    def,
                                    args,
                                    responder,
                                    trace_data,
                                } => match self.gen_frame_for(def, args) {
                                    Ok(next_frame) => {
                                        trace.root.inner.push(trace_data);
                                        stack.push(Frame::AwaitingNestedCall {
                                            interface_recv,
                                            handler,
                                            handler_fut,
                                            responder,
                                        });
                                        stack.push(next_frame);
                                    }
                                    Err(e) => {

                                        responder.send(Err(e)).unwrap();
                                        let recev_fut = interface_recv.clone().into_recv_async();
                                        stack.push(Frame::ReadyToPoll {
                                            interface_recv,
                                            recev_fut,
                                            handler,
                                            handler_fut,
                                        });
                                    }
                                }
                                BusInterfaceEvent::FwdError { error } => {
                                    todo!()
                                }
                            }
                        }
                        OneOfResult::F1(_, handler_return) => {
                            self.registered_stops.push(Arc::try_unwrap(handler).unwrap());
                            if stack.is_empty() {
                                trace.root.resolution = Some(Resolution::Success);
                                trace.root.return_v = Some(format!("{:#?}", handler_return.as_dbg()));
                                break 'main (Some(handler_return), trace);
                            } else {
                                if let Frame::AwaitingNestedCall {
                                    interface_recv,
                                    handler: returned_handler,
                                    handler_fut,
                                    responder,
                                } = stack.pop().unwrap()
                                {
                                    responder.send(Ok(handler_return)).unwrap();
                                    let recev_fut = interface_recv.clone().into_recv_async();
                                    stack.push(Frame::ReadyToPoll {
                                        interface_recv,
                                        handler: returned_handler,
                                        recev_fut,
                                        handler_fut,
                                    });
                                    continue 'main;
                                } else {
                                    unreachable!()
                                }
                            }
                        }
                        OneOfResult::All(..) => unreachable!(),
                    }
                }
                Frame::AwaitingNestedCall { .. } => unreachable!(),
            }
        }
    }

    pub async fn fire<
        Tag: unique_type::Unique,
        At: DynDebug + Sync + Send + 'static,
        Rt: DynDebug + Sync + Send + 'static,
    >(
        &mut self,
        def: &'static EventDef<Tag, At, Rt>,
        args: At,
    ) -> Result<Rt, CallTrace> {
        info!("Firing initial event: {:?}", def.name);
        let trace = CallTrace {
            root: CallEvent::from_event_def(def, &args),
        };
        let _ = def;
        let def = TypeId::of::<Tag>();
        let args = DynVar::new(args);
        match self.raw_fire(def, args, trace).await {
            (Some(return_v), _trace) => {
                Ok(return_v.try_to().unwrap())
            }
            (None, trace) => {
                Err(trace)
            }
        }
    }
}

impl Default for DABus {
    fn default() -> Self {
        Self::new()
    }
}
