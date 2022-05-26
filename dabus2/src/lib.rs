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
pub mod macros;
pub mod stop;
pub mod util;

pub use bus::DABus;
pub use event::{EventDef, EventRegister};
pub use interface::BusInterface;
pub use stop::BusStop;
