#![feature(drain_filter)]
#![allow(incomplete_features)]
#![feature(specialization)]

pub mod bus;
pub mod event;
pub mod interface;
pub mod prelude;
pub mod stop;
pub(crate) mod util;

#[macro_use]
extern crate log;
#[macro_use]
extern crate async_trait;

pub use bus::DABus;
pub use interface::BusInterface;
pub use stop::BusStop;
