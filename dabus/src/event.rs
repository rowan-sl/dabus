use std::any::{Any, TypeId};

use uuid::Uuid;

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

    pub fn into_raw(
        self,
    ) -> (Box<dyn Any + Send + 'static>, Box<dyn Any + Send + 'static>, Uuid) {
        (self.event, self.args, self.id)
    }

    pub fn uuid(&self) -> Uuid {
        self.id
    }
}
