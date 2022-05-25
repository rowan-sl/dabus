#![feature(downcast_unchecked)]
#![feature(drain_filter)]
#![allow(incomplete_features)]
#![feature(specialization)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate async_trait;
// #[macro_use]
// extern crate derivative;

pub mod bus;
pub mod core;
pub mod event;
pub mod interface;
pub mod stop;
pub mod util;
pub mod macros;

pub use event::{EventDef, EventRegister};
pub use bus::DABus;
pub use stop::BusStop;
pub use interface::BusInterface;

