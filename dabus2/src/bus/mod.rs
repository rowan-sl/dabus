pub mod error;

use core::any::TypeId;

use flume::{r#async::RecvFut, Receiver, Sender};
use futures::future::BoxFuture;

use crate::{
    core::dyn_var::DynVar,
    event::EventDef,
    interface::{BusInterface, BusInterfaceEvent},
    stop::{BusStopContainer, BusStopReq},
    util::{
        async_util::{OneOf, OneOfResult},
        dyn_debug::DynDebug,
    },
};
use error::{BaseFireEventError, FireEventError};

enum Frame {
    ReadyToPoll {
        interface_recv: Receiver<BusInterfaceEvent>,
        recev_fut: RecvFut<'static, BusInterfaceEvent>,
        handler_fut: BoxFuture<'static, (BusStopContainer, DynVar)>,
    },
    AwaitingNestedCall {
        interface_recv: Receiver<BusInterfaceEvent>,
        handler_fut: BoxFuture<'static, (BusStopContainer, DynVar)>,
        responder: Sender<Result<DynVar, FireEventError>>,
    },
}

pub struct DABus {
    registered_stops: Vec<BusStopContainer>,
}

impl DABus {
    pub const fn new() -> Self {
        Self {
            registered_stops: vec![],
        }
    }

    pub fn register<T: BusStopReq + Send + Sync + 'static>(&mut self, stop: T) {
        self.registered_stops
            .push(BusStopContainer::new(Box::new(stop)));
    }

    pub fn deregister<T: BusStopReq + Send + Sync + 'static>(&mut self) -> Option<T> {
        self.registered_stops
            .drain_filter(|stop| (*stop.inner).as_any().type_id() == TypeId::of::<T>())
            .nth(0)
            .map(|item| *item.inner.to_any().downcast().unwrap())
    }

    fn handlers_for(&mut self, def: TypeId) -> Vec<BusStopContainer> {
        self.registered_stops
            .drain_filter(|stop| stop.relevant(def))
            .collect()
    }

    fn gen_frame_for(&mut self, def: TypeId, args: DynVar) -> Result<Frame, FireEventError> {
        let mut handlers = self.handlers_for(def);
        assert!(
            handlers.len() < 2,
            "currently only supports one handler for an event! this WILL change soonTM"
        );
        if handlers.is_empty() {
            Err(FireEventError::from(BaseFireEventError::NoHandler))?
        }
        let handler = handlers.remove(0);
        let (interface_send, interface_recv): (Sender<BusInterfaceEvent>, _) = flume::bounded(1);
        let interface = BusInterface::new(interface_send);

        let recev_fut = interface_recv.clone().into_recv_async();
        let handler_fut = unsafe { handler.handle_raw_event(def, args, interface) };

        let frame = Frame::ReadyToPoll {
            interface_recv,
            recev_fut,
            handler_fut: Box::pin(async move { handler_fut.await }),
        };

        Ok(frame)
    }

    async fn raw_fire(&mut self, def: TypeId, args: DynVar) -> Result<DynVar, FireEventError> {
        let mut stack: Vec<Frame> = vec![];

        stack.push(self.gen_frame_for(def, args)?);

        'main: loop {
            match stack.pop().unwrap() {
                Frame::ReadyToPoll {
                    interface_recv,
                    recev_fut,
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
                                } => match self.gen_frame_for(def, args) {
                                    Ok(next_frame) => {
                                        stack.push(Frame::AwaitingNestedCall {
                                            interface_recv,
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
                                            handler_fut,
                                        });
                                    }
                                },
                            }
                        }
                        OneOfResult::F1(_, (handler, handler_return)) => {
                            if stack.is_empty() {
                                self.registered_stops.push(handler);
                                break 'main Ok(handler_return);
                            } else {
                                if let Frame::AwaitingNestedCall {
                                    interface_recv,
                                    handler_fut,
                                    responder,
                                } = stack.pop().unwrap()
                                {
                                    responder.send(Ok(handler_return)).unwrap();
                                    let recev_fut = interface_recv.clone().into_recv_async();
                                    stack.push(Frame::ReadyToPoll {
                                        interface_recv,
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
    ) -> Result<Rt, FireEventError> {
        let _ = def;
        let def = TypeId::of::<Tag>();
        let args = DynVar::new(args);
        Ok(unsafe { self.raw_fire(def, args).await?.try_to_unchecked::<Rt>() })
    }
}
