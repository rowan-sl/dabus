//! re-exports of all of `dabus`'s traits. all are re-exported as `_`,
//! so it does not clutter the namespace, but trait methods can still be used
//!
//! ***everone should make preludes like this!!!!!***

pub use crate::interface::BusErrorUtil as _;
pub use crate::stop::BusStop as _;
