#![feature(downcast_unchecked)]
#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(extract_if)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate async_trait;

pub mod bus;
pub(crate) mod core;
pub mod event;
pub(crate) mod interface;
pub(crate) mod macros;
pub(crate) mod stop;
pub(crate) mod util;
#[doc(hidden)]
pub mod unique_type;

#[doc(hidden)]
pub use ::concat_idents as __concat_idents;

pub use bus::{DABus, FireEvent};
pub use event::{EventDef, EventRegister};
pub use interface::{BusInterface, BusErrorUtil};
pub use stop::BusStop;

/// things that are just implementation details of the crate,
/// but might be nice to use (on a related topic to this crate)
///
/// do not expect (much) stability guarentees from this
pub mod extras {
    pub use crate::core::dyn_var::DynVar;
    pub use crate::util::{AsAny, PossiblyClone, TypeNamed, async_util, dyn_debug::DynDebug};
    pub use crate::event::async_fn_ptr;
}
