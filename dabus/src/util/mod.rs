//! the place where all of the very cursed workarounds for things like:
//! - cloning things that may not be clone
//! - getting the actual type name of dyn Any
//! - casting dyn T -> dyn Any (not as cursed)

pub mod possibly_clone;
pub mod dyn_typename;
pub mod dyn_downcast;

// all of these traits are implemeted for any T, so you dont have to explicitly require them
pub use possibly_clone::PossiblyClone;
pub use dyn_typename::TypeNamed;
pub use dyn_downcast::AsAny;

pub trait GeneralRequirements: TypeNamed + AsAny + 'static {}
impl<T: 'static> GeneralRequirements for T {}
