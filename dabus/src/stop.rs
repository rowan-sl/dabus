use std::{any::Any, fmt::Debug};

use crate::{event::BusEvent, interface::BusInterface};

#[async_trait::async_trait]
pub trait BusStop: Debug /* deal with it */ + Any /* i swear to god */ {
    async fn event(
        &mut self,
        event: &mut BusEvent,
        bus: BusInterface,
    ) -> BusEvent;

    fn cares(&mut self, event: &BusEvent) -> bool;
}
