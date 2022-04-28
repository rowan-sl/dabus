use std::any::{Any, TypeId};

use uuid::Uuid;


pub struct BusEvent {
    /// args the event was called with
    args: Option<Box<dyn Any + 'static>>,
    /// the event itself
    event: Option<Box<dyn Any + 'static>>,
    /// identifier used for event responses
    id: Uuid
}

impl BusEvent {
    pub fn new(event: impl Any + 'static, args: impl Any + 'static, id: Uuid) -> Self {
        Self {
            args: Some(Box::new(args)),
            event: Some(Box::new(event)),
            id,
        }
    }

    /// checks if the contained event type is of type `T`
    pub fn event_is<T: Any + 'static>(&self) -> bool {
        TypeId::of::<T>() == self.event.type_id() && self.event.is_some()
    }

    /// checks if the contained args are of type `T`
    pub fn args_are<T: Any + 'static>(&self) -> bool {
        TypeId::of::<T>() == self.args.type_id() && self.args.is_some()
    }

    /// if the contained event is of the type `E` and args are of type `A`, returning them bolth
    pub fn is_into<E: Any + 'static, A: Any + 'static>(&mut self) -> Option<(Box<E>, Box<A>)> {
        let event = self.event.take()?;
        let args = self.args.take()?;
        if !self.event_is::<E>() {
            self.event = Some(event);
            self.args = Some(args);
            return None;
        }
        if !self.args_are::<A>() {
            warn!("Mismatched args for event!");
            self.event = Some(event);
            self.args = Some(args);
            return None;
        }
        Some((event.downcast().unwrap(), args.downcast().unwrap()))
    }

    pub fn into_raw(mut self) -> Option<(Box<dyn Any + 'static>, Box<dyn Any + 'static>)> {
        let event = self.event.take()?;
        let args = self.args.take()?;
        Some((event, args))
    }

    pub fn uuid(&self) -> Uuid {
        self.id
    }
}
