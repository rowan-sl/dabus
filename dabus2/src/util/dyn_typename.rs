use std::any::{type_name, Any};

/// Allows getting the name of the type contained in a `Box<dyn Any>`
///
/// Care should be taken when using this, and the output should only be relied on for debugging
///
/// ## Pitfalls
/// make shure that you are only calling this on the type `&dyn Any` and **NO OTHER TYPE!!!!!**.
/// if called on Box<dyn Any>, it will return `std::box::Box<std::any::Any>` (or something similar)
/// instead of the type you want!
pub trait TypeNamed {
    /// gets the type name of Self
    fn type_name(&self) -> &'static str;
}

impl<T: Any + 'static> TypeNamed for T {
    fn type_name(&self) -> &'static str {
        type_name::<T>()
    }
}
