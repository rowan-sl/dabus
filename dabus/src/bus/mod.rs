//! the core of DABus

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
        local_trace_data: CallEvent,
    },
    AwaitingNestedCall {
        interface_recv: Receiver<BusInterfaceEvent>,
        handler: Arc<BusStopContainer>,
        handler_fut: BoxFuture<'static, DynVar>,
        responder: Sender<Result<DynVar, CallTrace>>,
        local_trace_data: CallEvent,
    },
}

/// Messaging bus and handler holder.
///
/// # Examples
///
/// registering an event handler
/// ```rust
/// # use dabus::{BusStop, EventRegister};
/// # #[derive(Debug, Default)]
/// # struct SomeEventHandler;
/// # impl BusStop for SomeEventHandler {
/// #     fn registered_handlers(h: EventRegister<Self>) -> EventRegister<Self> {
/// #        h
/// #     }
/// # }
/// #
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use dabus::DABus;
///
/// let mut bus = DABus::new();
/// bus.register(SomeEventHandler::default());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct DABus {
    registered_stops: Vec<BusStopContainer>,
}

impl DABus {
    /// Creates a new bus instance
    #[must_use]
    pub const fn new() -> Self {
        Self {
            registered_stops: vec![],
        }
    }

    /// Registers a handler with the bus, so that it can be used to handle events on this bus instance.
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

    /// Attempts to collect all handlers with the specified type and returns them. this is rather blunt,
    /// as there is no way of specifying a particular handler instance, but it is still usefull.
    pub fn deregister<T: BusStop + Debug + Send + Sync + 'static>(&mut self) -> Vec<T> {
        let stop = self
            .registered_stops
            .drain_filter(|stop| (*stop.inner.try_lock().unwrap()).as_any().type_id() == TypeId::of::<T>())
            .map(|item| *item.inner.into_inner().to_any().downcast().unwrap())
            .collect();
        stop
    }

    /// finds handlers for a specified event (def = TypeId of the tag type on a handler def)
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

    /// generates a new "stack frame" from a given event defeinition
    fn gen_frame_for(&mut self, def: TypeId, args: DynVar, mut local_trace_data: CallEvent) -> Result<Frame, CallEvent> {
        let mut handlers = self.handlers_for(def);
        assert!(
            handlers.len() < 2,
            "currently only supports one handler for an event! this WILL change soonTM"
        );
        if handlers.is_empty() {
            error!("no handlers found for {:?}", def);
            local_trace_data.resolve(Resolution::BusError(FireEventError::from(BaseFireEventError::NoHandler)));
            return Err(local_trace_data)
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
            local_trace_data,
        };

        Ok(frame)
    }

    /// the type-erased function that actually runs an event
    #[allow(clippy::too_many_lines)]// deal
    async fn raw_fire(&mut self, def: TypeId, args: DynVar, mut trace: CallTrace) -> (Option<DynVar>, CallTrace) {
        let mut stack: Vec<Frame> = vec![];

        stack.push(match self.gen_frame_for(def, args, trace.take_root().unwrap()) {
            Ok(initial_frame) => initial_frame,
            Err(initial_frame_error) => {
                trace.set_root(initial_frame_error);
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
                    mut local_trace_data,
                } => {
                    let recv_and_handler_fut = OneOf::new(recev_fut, handler_fut);
                    match recv_and_handler_fut.await {
                        OneOfResult::F0(interface_event, handler_fut) => {
                            info!("Received interface event: {:?}", interface_event);
                            match interface_event.unwrap() {
                                BusInterfaceEvent::Fire {
                                    def,
                                    args,
                                    responder,
                                    trace_data: next_event_trace_data,
                                } => match self.gen_frame_for(def, args, next_event_trace_data) {
                                    Ok(next_frame) => {
                                        stack.push(Frame::AwaitingNestedCall {
                                            interface_recv,
                                            handler,
                                            handler_fut,
                                            responder,
                                            local_trace_data,
                                        });
                                        stack.push(next_frame);
                                    }
                                    Err(error_trace) => {
                                        responder.send(Err(CallTrace {
                                            root: Some(error_trace)
                                        })).unwrap();
                                        let recev_fut = interface_recv.clone().into_recv_async();
                                        stack.push(Frame::ReadyToPoll {
                                            interface_recv,
                                            recev_fut,
                                            handler,
                                            handler_fut,
                                            local_trace_data,
                                        });
                                    }
                                }
                                BusInterfaceEvent::FwdBusError { mut error, blocker } => {
                                    drop(handler_fut);
                                    drop(blocker);
                                    let h = Arc::try_unwrap(handler).unwrap();
                                    self.registered_stops.push(h);
                                    local_trace_data.resolve(Resolution::NestedCallError);
                                    local_trace_data.push_inner(error.take_root().unwrap());
                                    if stack.is_empty() {
                                        trace.set_root(local_trace_data);
                                        break 'main (None, trace);
                                    } else if let Frame::AwaitingNestedCall {
                                        interface_recv,
                                        handler: nested_handler,
                                        handler_fut: nested_handler_fut,
                                        responder,
                                        local_trace_data: caller_handler_trace_data,
                                    } = stack.pop().unwrap() {
                                        responder.send(Err(CallTrace { root: Some(local_trace_data) })).unwrap();
                                        let recev_fut = interface_recv.clone().into_recv_async();
                                        stack.push(Frame::ReadyToPoll {
                                            interface_recv,
                                            handler: nested_handler,
                                            recev_fut,
                                            handler_fut: nested_handler_fut,
                                            local_trace_data: caller_handler_trace_data,
                                        });
                                    } else {
                                        unreachable!()
                                    }
                                }
                            }
                        }
                        OneOfResult::F1(_, handler_return) => {
                            info!("Handler returned");
                            self.registered_stops.push(Arc::try_unwrap(handler).unwrap());
                            if stack.is_empty() {
                                local_trace_data.resolve(Resolution::Success);
                                local_trace_data.set_return(&handler_return);
                                trace.set_root(local_trace_data);
                                break 'main (Some(handler_return), trace);
                            } else if let Frame::AwaitingNestedCall {
                                interface_recv,
                                handler: returned_handler,
                                handler_fut,
                                responder,
                                local_trace_data: mut caller_handler_trace_data,
                            } = stack.pop().unwrap()
                            {
                                local_trace_data.resolve(Resolution::Success);
                                local_trace_data.set_return(&handler_return);
                                caller_handler_trace_data.inner.push(local_trace_data);
                                responder.send(Ok(handler_return)).unwrap();
                                let recev_fut = interface_recv.clone().into_recv_async();
                                stack.push(Frame::ReadyToPoll {
                                    interface_recv,
                                    handler: returned_handler,
                                    recev_fut,
                                    handler_fut,
                                    local_trace_data: caller_handler_trace_data,
                                });
                            } else {
                                unreachable!()
                            }
                        }
                        OneOfResult::All(..) => unreachable!(),
                    }
                }
                Frame::AwaitingNestedCall { .. } => unreachable!(),
            }
        }
    }

    /// Fires an event on the bus, running appropreate handlers and returning the result.
    ///
    /// Args:
    /// - def: the definition of the event being triggered
    /// - args: arguments for the event handler
    ///
    /// # Returns
    ///
    /// on success, this returns the return value sent by the handler, as well as a call trace (this will change)
    ///
    /// on failure, this returns only the call trace, which can be used to find what went wrong
    ///
    /// # Panics
    ///
    /// if a handler that is called panics (or the runtime is broken)
    ///
    /// # Errors
    ///
    /// if there is some (expected) error with the runtime. currently this only includes not finding an appropreate handler
    ///
    pub async fn fire<Tag, At, Rt>(
        &mut self,
        def: &'static EventDef<Tag, At, Rt>,
        args: At,
    ) -> Result<FireEvent<Rt>, CallTrace>
    where
        Tag: unique_type::Unique,
        At: DynDebug + Sync + Send + 'static,
        Rt: DynDebug + Sync + Send + 'static,
    {
        info!("Firing initial event: {:?}", def.name);
        let trace = CallTrace {
            root: Some(CallEvent::from_event_def(def, &args)),
        };
        let _ = def;
        let def = TypeId::of::<Tag>();
        let args = DynVar::new(args);
        match self.raw_fire(def, args, trace).await {
            (Some(return_v), trace) => {
                Ok(FireEvent {
                    value: return_v.try_to().unwrap(),
                    trace,
                })
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

#[derive(Debug, Clone)]
pub struct FireEvent<T> {
    value: T,
    trace: CallTrace,
}

impl<T> FireEvent<T> {
    pub fn trace(&self) -> CallTrace {
        self.trace.clone()
    }

    pub fn ret(self) -> T {
        self.value
    }
}
