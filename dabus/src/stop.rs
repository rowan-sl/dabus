use std::{any::Any, fmt::Debug};

use crate::{
    event::{BusEvent, EventType},
    interface::BusInterface,
    util::{GeneralRequirements, PossiblyClone, TypeNamed},
};

/// Various ways that an event can be passed to a handler
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

/// Arg type pased to an event handler, containing the actual args.
///
/// the variants directly corrispond to the variants of [`EventActionType`]
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
}

impl<'a, T: PossiblyClone + Any + Send + 'static> EventArgs<'a, T> {
    /// convert `Self` into an owned `T` value
    ///
    /// # Panics
    /// when `Self` contains a reference instead of an owned value
    pub fn into_t(self) -> T {
        match self {
            Self::Consume(t) => t,
            Self::HandleCopy(t) => t,
            Self::HandleRef(..) => panic!("Called EventArgs::into_t on HandleRef variant!"),
        }
    }

    /// reutrens a reference to the value contained in `Self`
    ///
    /// infallible
    pub fn ref_t(&self) -> &T {
        match self {
            Self::Consume(t) => t,
            Self::HandleCopy(t) => t,
            Self::HandleRef(t) => t,
        }
    }

    /// returns if `Self` contains `&T` or `T`
    pub fn is_ref(&self) -> bool {
        match self {
            Self::Consume(..) => false,
            Self::HandleCopy(..) => false,
            Self::HandleRef(..) => true,
        }
    }
}

#[async_trait]
pub trait BusStop: Debug /* deal with it */ + Any /* i swear to god */ {
    /// the Event type passed to [`BusStop::event`]
    type Event: PossiblyClone + Any + Sync + Send + 'static;

    async fn event<'a>(
        &mut self,
        event: EventArgs<'a, Self::Event>,
        etype: EventType,
        bus: BusInterface,
    ) -> Option<Box<dyn GeneralRequirements + Send + 'static>>;//mabey make this a bit nicer/clearer what is supposed to be returned?

    /// after a type match check, how should this event be handled
    fn action(
        &mut self,
        event: &Self::Event,
    ) -> EventActionType;
}

#[async_trait]
pub(crate) trait BusStopMech: Debug + Any {
    async fn raw_event(
        &mut self,
        event: &mut Option<BusEvent>,
        etype: EventType,
        bus: BusInterface,
    ) -> Result<RawEventReturn, HandleEventError>;
    fn matches(&mut self, event: &BusEvent) -> bool;
    fn raw_action(&mut self, event: &BusEvent) -> EventActionType;
}

pub enum RawEventReturn {
    Response(BusEvent /* response */),
    // processed, but no response (send type event)
    Processed,
}

// watch the magic happen
#[async_trait]
impl<E, T> BusStopMech for T
where
    E: PossiblyClone + Any + Sync + Send + 'static,
    T: BusStop<Event = E> + Send,
{
    /// **IMPORTANT** make shure that the handlers are sorted by how they consume `event` before running them,
    /// and it should be an error if more than one tries to consume a event
    async fn raw_event(
        &mut self,
        // if this is None after raw_event is called, then the event is consumed and wont get to any other handler
        event: &mut Option<BusEvent>,
        etype: EventType,
        bus: BusInterface,
    ) -> Result<RawEventReturn, HandleEventError> {
        debug_assert!(event.is_some(), "Event state is not valid!");
        debug_assert!(self.matches(event.as_ref().unwrap()), "raw_event was called on a event that doesnt match!");

        let id = event.as_ref().unwrap().uuid();

        let event_args = match self.action(event.as_ref().unwrap().try_ref_event().unwrap()) {
            EventActionType::Consume => {
                let taken = event.take().unwrap();
                let event = taken.is_into::<E>().unwrap();
                EventArgs::Consume(*event)
            }
            EventActionType::HandleCopy => {
                let copy = event
                    .as_ref()
                    .unwrap()
                    .try_clone_event::<E>()
                    .expect("Event must be Clone in order to use HandleCopy");
                let event = copy.is_into::<E>().unwrap();
                EventArgs::HandleCopy(*event)
            }
            EventActionType::HandleRef => {
                let event = event.as_ref().unwrap().try_ref_event::<E>().unwrap();
                EventArgs::HandleRef(event)
            }
            EventActionType::Ignore => {
                return Err(HandleEventError::Ignored);
            }
        };

        match etype {
            EventType::Query => {
                match self
                    .event(event_args, EventType::Query, bus)
                    .await
                {
                    Some(response) => Ok(RawEventReturn::Response(BusEvent::new_raw(response, id))),
                    None => Err(HandleEventError::QueryNoResponse),
                }

            }
            EventType::Send => {
                let ret = self.event(event_args, EventType::Send, bus).await;
                if ret.is_some() {
                    Err(HandleEventError::SendSomeResponse(format!("Some({})", ret.unwrap().to_any().type_name())))
                } else {
                    Ok(RawEventReturn::Processed)
                }
            }
        }
    }

    fn matches(&mut self, event: &BusEvent) -> bool {
        event.event_is::<E>()
    }

    fn raw_action(&mut self, event: &BusEvent) -> EventActionType {
        self.action(event.try_ref_event().unwrap())
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum HandleEventError {
    #[error("Query events must have a response")]
    QueryNoResponse,
    #[error("Send events must not have a response\
    Expected `None`, found `{0}`\
    ")]
    SendSomeResponse(String),
    #[error("Event was ignored")]
    Ignored,
}
