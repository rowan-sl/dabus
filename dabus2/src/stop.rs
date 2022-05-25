use std::{any::TypeId, fmt::Debug};

use crate::{core::dyn_var::DynVar, event::EventRegister, interface::BusInterface};

pub trait BusStop {
    fn registered_handlers(h: EventRegister<Self>) -> EventRegister<Self>
    where
        Self: Sized;
}

mod seal {
    pub trait Sealed {}
}

#[async_trait]
pub trait BusStopMech: seal::Sealed {
    async unsafe fn handle_raw_event(&mut self, event_tag_id: TypeId, event: DynVar, interface: BusInterface) -> DynVar;
    fn relevant(&self, event_tag_id: TypeId) -> bool;
}

impl<T> seal::Sealed for T where T: BusStop + Sized + Send + Sync + 'static {}

#[async_trait]
impl<T> BusStopMech for T
where
    T: BusStop + Debug + Sized + Send + Sync + 'static,
{
    async unsafe fn handle_raw_event(
        &mut self,
        event_tag_id: TypeId,
        event: DynVar, /* must be the hidden event type */
        interface: BusInterface,
    ) -> DynVar /* the hidden return type */ {
        // TODO make this not query handlers each and every event
        let mut handlers = T::registered_handlers(EventRegister::new())
            .handlers
            .into_iter()
            .filter(|rh| rh.0 == event_tag_id)
            .collect::<Vec<_>>();
        debug_assert!(handlers.len() == 1);
        let handler = handlers.remove(0);

        let moved_self = std::ptr::read::<Self>(self as *mut Self as *const Self);
        let mut dyn_self = DynVar::new(moved_self);

        let fut = handler.1.call(&mut dyn_self, event, interface);
        let res = fut.await;

        let typed_self = dyn_self.try_to_unchecked::<Self>();
        std::ptr::write::<Self>(self as *mut Self, typed_self);

        res
    }

    fn relevant(&self, event_tag_id: TypeId) -> bool {
        // TODO make this not query handlers each and every event
        let handlers = T::registered_handlers(EventRegister::new())
            .handlers
            .into_iter()
            .filter(|rh| rh.0 == event_tag_id)
            .collect::<Vec<_>>();
        debug_assert!(handlers.len() <= 1);
        !handlers.is_empty()
    }
}
