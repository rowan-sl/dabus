use std::any::{type_name, Any, TypeId};

use uuid::Uuid;

use crate::util::{GeneralRequirements, PossiblyClone};

/// the type of event, ether Send or Query
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// an event that requires a response
    Query,
    /// a event that requires no response
    Send,
}

/// The raw representation of a event sent on the bus.
///
/// probably should not be exposed in a public interface?
#[derive(Derivative)]
#[derivative(Debug)]
pub struct BusEvent {
    /// the event itself
    #[derivative(Debug = "ignore")]
    event: Box<dyn GeneralRequirements + Send + 'static>,
    /// identifier used for event responses
    id: Uuid,
}

impl BusEvent {
    pub fn new(event: impl GeneralRequirements + Send + 'static, id: Uuid) -> Self {
        Self {
            event: Box::new(event),
            id,
        }
    }

    pub fn new_raw(event: Box<dyn GeneralRequirements + Send + 'static>, id: Uuid) -> Self {
        Self { event, id }
    }

    /// checks if the contained event type is of type `T`
    pub fn event_is<T: GeneralRequirements + Send + 'static>(&self) -> bool {
        let is = TypeId::of::<T>() == (*self.event).as_any().type_id();
        trace!(
            "Event: {}, is {}: {}",
            (*self.event).type_name(),
            type_name::<T>(),
            is
        );
        is
    }

    /// if the contained event is of the type `E` and args are of type `A`, returning them bolth
    pub fn is_into<E: GeneralRequirements + Send + 'static>(self) -> Result<Box<E>, Self> {
        if !self.event_is::<E>() {
            return Err(self);
        }
        let event = self.event;
        Ok(event.to_any().downcast().unwrap())
    }

    pub fn try_ref_event<'a, E: GeneralRequirements + Send + 'static>(
        &'a self,
    ) -> Result<&'a E, ()> {
        if !self.event_is::<E>() {
            return Err(());
        }
        match self.event.as_any().downcast_ref::<E>() {
            Some(a) => Ok(a),
            None => Err(()),
        }
    }

    pub fn into_raw(
        self,
    ) -> (
        Box<dyn GeneralRequirements + Send + 'static>,
        // Box<dyn Any + Send + 'static>,
        Uuid,
    ) {
        (self.event, self.id)
    }

    pub fn uuid(&self) -> Uuid {
        self.id
    }

    pub fn try_clone_event<E: GeneralRequirements + Any + Send + 'static>(
        &self,
    ) -> Result<Self, ()> {
        let new_event = match self.event.as_any().downcast_ref::<E>() {
            Some(event) => {
                if event.is_clone() {
                    event.try_clone()
                } else {
                    return Err(());
                }
            }
            None => return Err(()),
        };

        Ok(Self {
            event: Box::new(new_event),
            id: self.id.clone(),
        })
    }

    pub fn map_fn_if<A: GeneralRequirements + Send + 'static, B, F: FnOnce(A) -> B>(
        &self,
        map_fn: F,
    ) -> Option<impl FnOnce(Self) -> B> {
        if self.event_is::<A>() && (*self.event).type_id() == TypeId::of::<A>() {
            Some(move |event: Self| -> B { map_fn(*event.is_into::<A>().unwrap()) })
        } else {
            None
        }
    }
}
