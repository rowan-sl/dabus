//! the place where all of the very cursed workarounds for things like:
//! - cloning things that may not be clone
//! - getting the actual type name of dyn Any
//! - casting dyn T -> dyn Any (not as cursed)

pub mod async_util;
pub mod dyn_downcast;
pub mod dyn_typename;
pub mod possibly_clone;
pub mod dyn_debug;

// all of these traits are implemeted for any T, so you dont have to explicitly require them
pub use dyn_downcast::AsAny;
pub use dyn_typename::TypeNamed;
pub use possibly_clone::PossiblyClone;

use self::dyn_debug::DynDebug;

/// convenience trait for [`TypeNamed`] + [`AsAny`] + 'static
pub trait GeneralRequirements: DynDebug + TypeNamed + AsAny + 'static {}
impl<T: DynDebug + 'static> GeneralRequirements for T {}
