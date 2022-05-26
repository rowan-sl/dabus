pub mod error;

use core::any::TypeId;

use flume::Sender;

use crate::{core::dyn_var::DynVar, event::EventDef, stop::{BusStopContainer, BusStopReq}, util::dyn_debug::DynDebug, interface::{BusInterface, BusInterfaceEvent}};
use error::{FireEventError, BaseFireEventError};



pub struct DABus {
    registered_stops: Vec<BusStopContainer>,
}

impl DABus {
    pub const fn new() -> Self {
        Self {
            registered_stops: vec![],
        }
    }

    pub fn register<T: BusStopReq>(&mut self, stop: T) {
        self.registered_stops.push(BusStopContainer::new(Box::new(stop)));
    }

    pub fn deregister<T: BusStopReq>(&mut self) -> Option<T> {
        self.registered_stops
            .drain_filter(|stop| (*stop.inner).as_any().type_id() == TypeId::of::<T>())
            .nth(0)
            .map(|item| *item.inner.to_any().downcast().unwrap())
    }

    fn handlers_for<Tag: unique_type::Unique, At, Rt>(
        &mut self,
        def: &'static EventDef<Tag, At, Rt>,
    ) -> Vec<BusStopContainer> {
        let _ = def; // here for seminatics
        self.registered_stops
            .drain_filter(|stop| stop.relevant(TypeId::of::<Tag>()))
            .collect()
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
        let mut handlers = self.handlers_for(def);
        assert!(
            handlers.len() < 2,
            "currently only supports one handler for an event! this WILL change soonTM"
        );

        if handlers.is_empty() {
            Err(FireEventError::from(BaseFireEventError::NoHandler))?
        }

        let (interface_send, _interface_recv): (Sender<BusInterfaceEvent>, _) = flume::unbounded();
        // currently only for design use, no functionality yet
        let interface = BusInterface::new(interface_send);

        let handler = handlers.remove(0);
        Ok(unsafe {
            let (handler, result) = handler
                .handle_raw_event(TypeId::of::<Tag>(), DynVar::new(args), interface)
                .await;
            self.registered_stops.push(handler);
            result.try_to_unchecked::<Rt>()
        })

    }
}
