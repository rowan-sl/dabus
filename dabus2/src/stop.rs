use std::{any::TypeId, fmt::Debug};

use crate::{core::dyn_var::DynVar, event::EventRegister};

pub trait BusStop {
    type Event;
    fn events(h: EventRegister<<Self as BusStop>::Event>) -> EventRegister<<Self as BusStop>::Event>;
}

mod seal {
    pub trait Sealed {}
}

#[async_trait]
pub trait BusStopMech: seal::Sealed {
    async unsafe fn handle_raw_event(&mut self, event_tag_id: TypeId, event: DynVar) -> DynVar;
    fn relevant(&self, event_tag_id: TypeId) -> bool;
}

impl<T> seal::Sealed for T where T: BusStop + Sized + Send + Sync + 'static {}

// #[async_trait]
// impl<T> BusStopMech for T
// where
//     T: BusStop + Debug + Sized + Send + Sync + 'static,
// {
//     async unsafe fn handle_raw_event(
//         &mut self,
//         event_tag_id: TypeId,
//         event: DynVar, /* must be the hidden event type */
//     ) -> DynVar /* the hidden return type */ {
//         // TODO make this not query handlers each and every event
//         let mut handlers = T::registered_handlers(EventRegister::new())
//             .handlers
//             .into_iter()
//             .filter(|rh| rh.0 == event_tag_id)
//             .collect::<Vec<_>>();
//         debug_assert!(handlers.len() == 1);
//         let handler = handlers.remove(0);
//         // let fut = handler.1.call_erased(self, event);
//         // fut.await
//         todo!()
//     }

//     fn relevant(&self, event_tag_id: TypeId) -> bool {
//         // TODO make this not query handlers each and every event
//         let handlers = T::registered_handlers(EventRegister::new())
//             .handlers
//             .into_iter()
//             .filter(|rh| rh.0 == event_tag_id)
//             .collect::<Vec<_>>();
//         debug_assert!(handlers.len() <= 1);
//         !handlers.is_empty()
//     }
// }

// VERY unsafe
unsafe fn detach<'a, 'b, T>(x: &'a mut T) -> &'b mut T {
    &mut *(x as *mut T)
}
