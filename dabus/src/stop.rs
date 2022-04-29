use std::{any::Any, fmt::Debug};

use crate::{event::BusEvent, interface::BusInterface};

#[async_trait]
pub trait BusStop: Debug /* deal with it */ + Any /* i swear to god */ {
    type Event: Any + Send + 'static;
    type Args: Any + Send + 'static;
    type Response: Any + Send + 'static;

    async fn event(
        &mut self,
        event: Self::Event,
        args: Self::Args,
        bus: BusInterface,
    ) -> Self::Response;
}

#[async_trait]
pub(crate) trait BusStopMech: Debug + Any {
    async fn raw_event(&mut self, event: &mut BusEvent, bus: BusInterface) -> BusEvent;
    fn cares(&mut self, event: &BusEvent) -> bool;
}

// watch the magic happen
#[async_trait]
impl<E, A, R, T> BusStopMech for T
where
    E: Any + Send + 'static,
    A: Any + Send + 'static,
    R: Any + Send + 'static,
    T: BusStop<Event = E, Args = A, Response = R> + Send,
{
    async fn raw_event(&mut self, event: &mut BusEvent, bus: BusInterface) -> BusEvent {
        let id = event.uuid();
        let (event, args) = event.is_into::<E, A>().unwrap();

        let response = self.event(*event, *args, bus).await;

        BusEvent::new(crate::bus::sys::ReturnEvent, response, id)
    }

    fn cares(&mut self, event: &BusEvent) -> bool {
        event.event_is::<E>() & event.args_are::<A>()
    }
}
