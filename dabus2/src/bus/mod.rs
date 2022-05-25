pub mod error;

use core::any::TypeId;

use flume::Sender;

use crate::{core::dyn_var::DynVar, event::EventDef, stop::BusStopMech, util::{GeneralRequirements, dyn_debug::DynDebug}, interface::{BusInterface, BusInterfaceEvent}};

pub trait BusStopReq: BusStopMech + GeneralRequirements {}
impl<T: BusStopMech + GeneralRequirements> BusStopReq for T {}

pub struct DABus {
    registered_stops: Vec<Box<dyn BusStopReq + 'static>>,
}

impl DABus {
    pub const fn new() -> Self {
        Self {
            registered_stops: vec![],
        }
    }

    pub fn register<T: BusStopReq>(&mut self, stop: T) {
        self.registered_stops.push(Box::new(stop));
    }

    pub fn deregister<T: BusStopReq>(&mut self) -> Option<T> {
        self.registered_stops
            .drain_filter(|stop| (*stop).as_any().type_id() == TypeId::of::<T>())
            .nth(0)
            .map(|item| *item.to_any().downcast().unwrap())
    }

    fn handlers_for<Tag: unique_type::Unique, At, Rt>(
        &mut self,
        def: &'static EventDef<Tag, At, Rt>,
    ) -> Vec<Box<dyn BusStopReq + 'static>> {
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
    ) -> Rt {
        let mut handlers = self.handlers_for(def);
        assert!(
            handlers.len() < 2,
            "currently only supports one handler for an event! this WILL change soonTM"
        );
        assert!(!handlers.is_empty(), "no handler matches the event");

        let (interface_send, _interface_recv): (Sender<BusInterfaceEvent>, _) = flume::unbounded();
        // currently only for design use, no functionality yet
        let interface = BusInterface::new(interface_send);

        let mut handler = handlers.remove(0);
        let result = unsafe {
            handler
                .handle_raw_event(TypeId::of::<Tag>(), DynVar::new(args), interface)
                .await
                .try_to::<Rt>()
                .unwrap()
        };
        self.registered_stops.push(handler);
        result
    }
}
