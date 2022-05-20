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
}

#[async_trait]
pub trait BusStop: Debug /* deal with it */ + Any /* i swear to god */ {
    /// the Event type passed to [`BusStop::event`]
    type Event: PossiblyClone + Any + Sync + Send + 'static;

    /// Called when an event occurs
    ///
    /// ...
    ///
    /// ...
    ///
    /// *crickets*
    ///
    /// ...
    ///
    /// this needs better documentation
    async fn event(
        &mut self,
        event: Self::Event,
        bus: BusInterface,
    ) -> Option<Box<dyn GeneralRequirements + Send + 'static>>;//mabey make this a bit nicer/clearer what is supposed to be returned?

    /// Checks if a **send type** event is relevant to the current function,
    /// returning a conversion function and the handling method
    ///
    /// may be called multiple times for one event, and MUST return the same each time
    ///
    /// # Returns
    /// if the event is relevant to the function, returns Some with the method to handle it, if it is not relevant, returns None
    ///
    /// # Purity
    /// this function should not depend on external state,
    /// or modify local state (output MUST be repeateable if called multiple times)
    fn map_shared_event(
        &self,
        event: &BusEvent,
    ) -> Option<(Box<dyn FnOnce(BusEvent) -> Self::Event>, EventActionType)>;
}

#[async_trait]
pub(crate) trait BusStopMech: Debug + Any {
    async fn raw_event(
        &mut self,
        event: &mut Option<BusEvent>,
        etype: EventType,
        bus: BusInterface,
    ) -> Result<RawEventReturn, HandleEventError>;

    #[must_use]
    fn raw_action(&mut self, event: &BusEvent, etype: EventType) -> RawAction;
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

        let id = event.as_ref().unwrap().uuid();

        let event_args =
            match self.raw_action(event.as_ref().unwrap().try_ref_event().unwrap(), etype) {
                RawAction::NoConversion | RawAction::TypeMismatch => unreachable!(),
                RawAction::QueryEvent => {
                    let taken = event.take().unwrap();
                    let event = taken.is_into::<E>().unwrap();
                    *event
                }
                RawAction::SendEvent(atype) => {
                    let cvt_fn = self.map_shared_event(event.as_ref().unwrap()).unwrap().0;
                    match atype {
                        EventActionType::Consume => cvt_fn(event.take().unwrap()),
                        EventActionType::HandleCopy => cvt_fn(
                            event
                                .as_ref()
                                .unwrap()
                                .try_clone_event::<E>()
                                .expect("Event must be Clone in order to use HandleCopy"),
                        ),
                    }
                }
            };

        match etype {
            EventType::Query => match self.event(event_args, bus).await {
                Some(response) => Ok(RawEventReturn::Response(BusEvent::new_raw(response, id))),
                None => Err(HandleEventError::QueryNoResponse),
            },
            EventType::Send => {
                let ret = self.event(event_args, bus).await;
                if ret.is_some() {
                    Err(HandleEventError::SendSomeResponse(format!(
                        "Some({})",
                        ret.unwrap().to_any().type_name()
                    )))
                } else {
                    Ok(RawEventReturn::Processed)
                }
            }
        }
    }

    #[must_use]
    fn raw_action(&mut self, event: &BusEvent, etype: EventType) -> RawAction {
        match etype {
            EventType::Query => RawAction::QueryEvent,
            EventType::Send => {
                if let Some((_, acttype)) = self.map_shared_event(event) {
                    RawAction::SendEvent(acttype)
                } else {
                    RawAction::NoConversion
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawAction {
    /// failure of type check
    TypeMismatch,
    /// failure of Send event maping
    NoConversion,
    /// success, event type is Query (EventActionType::Consume is implied)
    QueryEvent,
    /// success, event type is Send, and event action is provided
    SendEvent(EventActionType),
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum HandleEventError {
    #[error("Query events must have a response")]
    QueryNoResponse,
    #[error(
        "Send events must not have a response\
    Expected `None`, found `{0}`\
    "
    )]
    SendSomeResponse(String),
}
