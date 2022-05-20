//! the place where all of the very cursed workarounds for things like:
//! - cloning things that may not be clone
//! - getting the actual type name of dyn Any
//! - casting dyn T -> dyn Any (not as cursed)

pub mod async_util;
pub mod dyn_downcast;
pub mod dyn_typename;
pub mod possibly_clone;

// all of these traits are implemeted for any T, so you dont have to explicitly require them
pub use dyn_downcast::AsAny;
pub use dyn_typename::TypeNamed;
pub use possibly_clone::PossiblyClone;

/// convenience trait for [`TypeNamed`] + [`AsAny`] + 'static
pub trait GeneralRequirements: ::core::fmt::Debug + TypeNamed + AsAny + 'static {}
impl<T: ::core::fmt::Debug + 'static> GeneralRequirements for T {}
