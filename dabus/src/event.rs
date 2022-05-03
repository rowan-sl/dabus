use std::any::{Any, TypeId};

use uuid::Uuid;

use crate::util::possibly_clone::PossiblyClone;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// an event that requires a response
    Query,
    /// a event that requires no response
    Send,
}

#[derive(Debug)]
pub struct BusEvent {
    /// args the event was called with
    args: Box<dyn Any + Send + 'static>,
    /// the event itself
    event: Box<dyn Any + Send + 'static>,
    /// identifier used for event responses
    id: Uuid,
}

impl BusEvent {
    pub fn new(
        event: impl Any + Send + 'static,
        args: impl Any + Send + 'static,
        id: Uuid,
    ) -> Self {
        Self {
            args: Box::new(args),
            event: Box::new(event),
            id,
        }
    }

    /// checks if the contained event type is of type `T`
    pub fn event_is<T: Any + Send + 'static>(&self) -> bool {
        TypeId::of::<T>() == (*self.event).type_id()
    }

    /// checks if the contained args are of type `T`
    pub fn args_are<T: Any + Send + 'static>(&self) -> bool {
        TypeId::of::<T>() == (*self.args).type_id()
    }

    /// if the contained event is of the type `E` and args are of type `A`, returning them bolth
    pub fn is_into<E: Any + Send + 'static, A: Any + Send + 'static>(
        self,
    ) -> Result<(Box<E>, Box<A>), Self> {
        // trace!("Attempting to convert to the type {} {}", type_name::<E>(), type_name::<A>());
        if !self.event_is::<E>() {
            // trace!("is_into: event mismatch");
            return Err(self);
        }
        if !self.args_are::<A>() {
            warn!("Mismatched args for event!");
            return Err(self);
        }
        let event = self.event;
        let args = self.args;
        Ok((event.downcast().unwrap(), args.downcast().unwrap()))
    }

    pub fn try_ref_args<'a, A: Any + Send + 'static>(&'a self) -> Result<&'a A, ()> {
        if !self.args_are::<A>() {
            return Err(());
        }
        match self.args.downcast_ref::<A>() {
            Some(a) => Ok(a),
            None => Err(()),
        }
    }

    pub fn into_raw(
        self,
    ) -> (
        Box<dyn Any + Send + 'static>,
        Box<dyn Any + Send + 'static>,
        Uuid,
    ) {
        (self.event, self.args, self.id)
    }

    pub fn uuid(&self) -> Uuid {
        self.id
    }

    pub fn clone_event<E: Clone + Any + Send + 'static>(&self) -> Result<E, ()> {
        match self.event.downcast_ref::<E>() {
            Some(event) => Ok(event.clone()),
            None => Err(()),
        }
    }

    pub fn try_clone_event<
        E: Clone + Any + Send + 'static,
        A: PossiblyClone + Any + Send + 'static,
    >(
        &self,
    ) -> Result<Self, ()> {
        let new_event = match self.event.downcast_ref::<E>() {
            Some(event) => event.clone(),
            None => return Err(()),
        };

        let new_args = match self.event.downcast_ref::<A>() {
            Some(event) => {
                if A::IS_CLONE {
                    event.try_clone()
                } else {
                    return Err(());
                }
            }
            None => return Err(()),
        };

        Ok(Self {
            event: Box::new(new_event),
            args: Box::new(new_args),
            id: self.id.clone(),
        })
    }
}
