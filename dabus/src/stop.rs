use std::{any::Any, fmt::Debug};

use crate::{event::BusEvent, BusInterface};

pub trait BusStop: Debug /* deal with it */ + Any /* i swear to god */ {
    fn event(
        &mut self,
        event: &mut BusEvent,
        bus: BusInterface,
    );
}
