use std::any::{Any, TypeId};

use uuid::Uuid;

pub struct BusEvent {
    /// args the event was called with
    args: Option<Box<dyn Any + Send + 'static>>,
    /// the event itself
    event: Option<Box<dyn Any + Send + 'static>>,
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
            args: Some(Box::new(args)),
            event: Some(Box::new(event)),
            id,
        }
    }

    /// checks if the contained event type is of type `T`
    pub fn event_is<T: Any + Send + 'static>(&self) -> bool {
        let is = if let Some(ref event) = self.event {
            TypeId::of::<T>() == (**event).type_id()
        } else {
            false
        };
        // trace!("event is {}: {}", type_name::<T>(), is);
        is
    }

    /// checks if the contained args are of type `T`
    pub fn args_are<T: Any + Send + 'static>(&self) -> bool {
        let are = if let Some(ref args) = self.args {
            TypeId::of::<T>() == (**args).type_id()
        } else {
            false
        };
        // trace!("args are {}: {}", type_name::<T>(), are);
        are
    }

    /// if the contained event is of the type `E` and args are of type `A`, returning them bolth
    pub fn is_into<E: Any + Send + 'static, A: Any + Send + 'static>(
        &mut self,
    ) -> Option<(Box<E>, Box<A>)> {
        // trace!("Attempting to convert to the type {} {}", type_name::<E>(), type_name::<A>());
        if !self.event_is::<E>() {
            // trace!("is_into: event mismatch");
            return None;
        }
        if !self.args_are::<A>() {
            warn!("Mismatched args for event!");
            return None;
        }
        let event = self.event.take()?;
        let args = self.args.take()?;
        Some((event.downcast().unwrap(), args.downcast().unwrap()))
    }

    pub fn into_raw(
        mut self,
    ) -> Option<(Box<dyn Any + Send + 'static>, Box<dyn Any + Send + 'static>)> {
        let event = self.event.take()?;
        let args = self.args.take()?;
        Some((event, args))
    }

    pub fn uuid(&self) -> Uuid {
        self.id
    }
}
