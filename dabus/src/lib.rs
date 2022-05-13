#![feature(drain_filter)]
#![allow(incomplete_features)]
#![feature(specialization)]

pub mod args;
pub mod bus;
pub mod event;
pub mod interface;
pub mod prelude;
pub mod stop;
pub mod util;

#[allow(unused_imports)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate derivative;

pub use bus::DABus;
pub use interface::BusInterface;
pub use stop::BusStop;
