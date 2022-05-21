use std::any::TypeId;

use crate::{core::dyn_var::DynVar, event::Handlers};

pub trait BusStop {
    fn registered_handlers(h: Handlers<Self>) -> Handlers<Self>
    where
        Self: Sized;
}

pub(crate) trait BusStopMech {
    fn handle_raw_event(&mut self, event_tag_id: TypeId, event: DynVar) -> DynVar;
}

// impl<T> BusStopMech for T
// where
//     T: BusStop + Sized + 'static,
// {
//     fn handle_raw_event(
//         &mut self,
//         event_tag_id: TypeId,
//         event: DynVar, /* must be the hidden event type */
//     ) -> DynVar /* the hidden return type */ {
//         // TODO make this not query handlers each and every event
//         let mut handlers = T::registered_handlers(Handlers::new())
//             .handlers
//             .into_iter()
//             .filter(|rh| unsafe { rh.releavant_to(event_tag_id) })
//             .collect::<Vec<_>>();
//         debug_assert!(handlers.len() == 1);
//         let handler = handlers.remove(0);
//         unsafe { handler.call(self, event) }
//     }
// }
