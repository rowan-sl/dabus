use std::any::{Any, TypeId};

use uuid::Uuid;

use crate::util::PossiblyClone;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// an event that requires a response
    Query,
    /// a event that requires no response
    Send,
}

#[derive(Debug)]
pub struct BusEvent {
    /// the event itself
    event: Box<dyn Any + Send + 'static>,
    /// identifier used for event responses
    id: Uuid,
}

impl BusEvent {
    pub fn new(
        event: impl Any + Send + 'static,
        id: Uuid,
    ) -> Self {
        Self {
            event: Box::new(event),
            id,
        }
    }

    pub fn new_raw(
        event: Box<dyn Any + Send + 'static>,
        id: Uuid,
    ) -> Self {
        Self {
            event,
            id,
        }
    }

    /// checks if the contained event type is of type `T`
    pub fn event_is<T: Any + Send + 'static>(&self) -> bool {
        TypeId::of::<T>() == (*self.event).type_id()
    }

    /// checks if the contained args are of type `T`
    // pub fn args_are<T: Any + Send + 'static>(&self) -> bool {
    //     TypeId::of::<T>() == (*self.args).type_id()
    // }

    /// if the contained event is of the type `E` and args are of type `A`, returning them bolth
    pub fn is_into<E: Any + Send + 'static>(
        self,
    ) -> Result<Box<E>, Self> {
        // trace!("Attempting to convert to the type {} {}", type_name::<E>(), type_name::<A>());
        if !self.event_is::<E>() {
            // trace!("is_into: event mismatch");
            return Err(self);
        }
        // if !self.args_are::<A>() {
        //     warn!("Mismatched args for event!");
        //     return Err(self);
        // }
        let event = self.event;
        Ok(event.downcast().unwrap())
    }

    pub fn try_ref_event<'a, E: Any + Send + 'static>(&'a self) -> Result<&'a E, ()> {
        if !self.event_is::<E>() {
            return Err(());
        }
        match self.event.downcast_ref::<E>() {
            Some(a) => Ok(a),
            None => Err(()),
        }
    }

    pub fn into_raw(
        self,
    ) -> (
        Box<dyn Any + Send + 'static>,
        // Box<dyn Any + Send + 'static>,
        Uuid,
    ) {
        (self.event, self.id)
    }

    pub fn uuid(&self) -> Uuid {
        self.id
    }

    // pub fn clone_event<E: Clone + Any + Send + 'static>(&self) -> Result<E, ()> {
    //     match self.event.downcast_ref::<E>() {
    //         Some(event) => Ok(event.clone()),
    //         None => Err(()),
    //     }
    // }

    pub fn try_clone_event<
        E: PossiblyClone + Any + Send + 'static,
    >(
        &self,
    ) -> Result<Self, ()> {
        let new_event = match self.event.downcast_ref::<E>() {
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
}
