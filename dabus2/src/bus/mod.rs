use crate::{stop::BusStopMech, util::GeneralRequirements};
use core::any::TypeId;

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
}
