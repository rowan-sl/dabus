use std::{any::Any, fmt::Debug};

use crate::{
    bus::sys::ReturnEvent,
    event::{BusEvent, EventType},
    interface::BusInterface,
    util::possibly_clone::PossiblyClone,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventActionType {
    /// consume the event.
    ///
    /// the handler is given the original unique copy of the event, and no clone is made (can be run on non-clone events)
    ///
    /// if multiple handlers request to consume an event, you should NOT rely on any particular handler to get priority
    Consume,
    /// take a copy of the event, the original event can be passed on to other handlers
    HandleCopy,
    /// take a reference to the event,
    HandleRef,
    /// do not handle the event
    Ignore,
}

#[derive(Debug)]
pub enum EventArgs<'a, T: PossiblyClone + Any + Send + 'static> {
    /// consume the event.
    ///
    /// the handler is given the original unique copy of the event, and no clone is made (can be run on non-clone events)
    ///
    /// if multiple handlers request to consume an event, you should NOT rely on any particular handler to get priority
    Consume(T),
    /// take a copy of the event, the original event can be passed on to other handlers
    HandleCopy(T),
    /// take a reference to the event,
    HandleRef(&'a T),
    /// do not handle the event
    Ignore,
}

#[async_trait]
pub trait BusStop: Debug /* deal with it */ + Any /* i swear to god */ {
    type Event: Clone + Any + Send + 'static;
    type Args: PossiblyClone + Any + Send + 'static;
    type Response: Any + Send + 'static;

    /// handle a query-type event
    async fn query_event<'a>(
        &mut self,
        args: EventArgs<'a, Self::Args>,
        bus: BusInterface,
    ) -> Self::Response;

    /// handle a send-type event
    async fn send_event<'a>(
        &mut self,
        args: EventArgs<'a, Self::Args>,
        bus: BusInterface,
    );

    /// after a type match check, how should this event be handled
    fn action(
        &mut self,
        event: Self::Event,
        etype: EventType,
    ) -> EventActionType;
}

#[async_trait(?Send)]
pub(crate) trait BusStopMech: Debug + Any {
    async fn raw_event(
        &mut self,
        event: &mut Option<BusEvent>,
        etype: EventType,
        bus: BusInterface,
    ) -> RawEventReturn;
    fn matches(&mut self, event: &BusEvent) -> bool;
    fn raw_action(&mut self, event: &BusEvent, etype: EventType) -> EventActionType;
}

pub enum RawEventReturn {
    Ignored,
    Response(BusEvent /* response */),
    // processed, but no response (send type event)
    Processed,
}

// watch the magic happen
#[async_trait(?Send)]
impl<E, A, R, T> BusStopMech for T
where
    E: Clone + Any + Send + 'static,
    A: PossiblyClone + Any + Send + 'static,
    R: Any + Send + 'static,
    T: BusStop<Event = E, Args = A, Response = R> + Send,
{
    /// **IMPORTANT** make shure that the handlers are sorted by how they consume `event` before running them,
    /// and it should be an error if more than one tries to consume a event
    async fn raw_event(
        &mut self,
        // if this is None after raw_event is called, then the event is consumed and wont get to any other handler
        event: &mut Option<BusEvent>,
        etype: EventType,
        bus: BusInterface,
    ) -> RawEventReturn {
        assert!(event.is_some());
        assert!(self.matches(event.as_ref().unwrap()));

        let id = event.as_ref().unwrap().uuid();

        let event_args = match self.action(event.as_ref().unwrap().clone_event().unwrap(), etype) {
            EventActionType::Consume => {
                let taken = event.take().unwrap();
                let (_, args) = taken.is_into::<E, A>().unwrap();
                EventArgs::Consume(*args)
            }
            EventActionType::HandleCopy => {
                let copy = event
                    .as_ref()
                    .unwrap()
                    .try_clone_event::<E, A>()
                    .expect("Event must be Clone in order to use HandleCopy");
                let (_, args) = copy.is_into::<E, A>().unwrap();
                EventArgs::HandleCopy(*args)
            }
            EventActionType::HandleRef => {
                let args = event.as_ref().unwrap().try_ref_args::<A>().unwrap();
                EventArgs::HandleRef(args)
            }
            EventActionType::Ignore => {
                return RawEventReturn::Ignored;
            }
        };

        match etype {
            EventType::Query => {
                let response = self.query_event(event_args, bus).await;
                RawEventReturn::Response(BusEvent::new(ReturnEvent, response, id))
            }
            EventType::Send => {
                self.send_event(event_args, bus).await;
                RawEventReturn::Processed
            }
        }
    }

    fn matches(&mut self, event: &BusEvent) -> bool {
        event.event_is::<E>() & event.args_are::<A>()
    }

    fn raw_action(&mut self, event: &BusEvent, etype: EventType) -> EventActionType {
        self.action(event.clone_event().unwrap(), etype)
    }
}
