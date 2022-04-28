use std::any::{Any, TypeId};

pub struct BusEvent(Option<Box<dyn Any + 'static>>);

impl BusEvent {
    pub fn new(event: impl Any + 'static) -> Self {
        Self(Some(Box::new(event)))
    }

    pub(crate) fn consumed(&self) -> bool {
        self.0.is_none()
    }

    /// checks if the contained value exists, and is of the type `T`
    pub fn is<T: Any + 'static>(&self) -> bool {
        TypeId::of::<T>() == self.0.type_id() && self.0.is_some()
    }

    /// if the contained event is of the type `T`, then it returns the event
    pub fn is_into<T: Any + 'static>(&mut self) -> Option<Box<T>> {
        let raw = self.0.take()?;
        match raw.downcast::<T>() {
            Ok(event) => Some(event),
            Err(mismatched) => {
                self.0 = Some(mismatched);
                None
            }
        }
    }
}
