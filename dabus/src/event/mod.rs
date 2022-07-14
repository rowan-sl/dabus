//! event declaration related things

pub mod async_fn_ptr;

use std::{
    any::{type_name, TypeId},
    marker::PhantomData,
};

pub use unique_type;

use self::async_fn_ptr::{AsyncFnPtr, HandlerCallableErased, HandlerFn};

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct EventDef {
    pub(crate) handler_name: &'static str,
    pub(crate) name: &'static str,
    pub(crate) at: TypeId,
    pub(crate) rt: TypeId,
}

impl EventDef {
    #[doc(hidden)]
    #[must_use]
    pub const fn new<At: 'static, Rt: 'static>(handler_name: &'static str, name: &'static str) -> Self {
        Self { handler_name, name, at: TypeId::of::<At>(), rt: TypeId::of::<Rt>() }
    }
}

#[macro_export]
macro_rules! bus_stop_impl {
    ($struct:ty, $ev_var:ident : $( ($name:ident, $at:ty, $rt:ty) ),*) => {
        // expansion: $( $names ),*
        // {
        const $ev_var: &'static [$crate::event::EventDef] = [ $( $crate::event::EventDef::new::<$at, $rt>(stringify!($struct), stringify!($name)), ),* ].as_slice();
        $( const $name: &'static $crate::event::EventDef = &$crate::event::EventDef::new::<$at, $rt>(stringify!($struct), stringify!($name)); ),*
        // }
    };
}

// /// type for declaring events.
// ///
// /// example:
// /// ```rust
// /// use dabus::event;
// ///
// /// event!(TEST_EVENT, (), ());
// /// ```
// #[allow(clippy::module_name_repetitions)]
// pub struct EventDef<
//     Tag: unique_type::Unique,
//     At,
//     Rt = (), /* if At is `()`, than this event is eligeble for lazy evaluation */
// > {
//     pub(crate) name: &'static str,
//     _tag: PhantomData<*const Tag /* dropck */>,
//     _at: PhantomData<*const At /* also dropck */>,
//     _rt: PhantomData<*const Rt /* also dropck */>,
// }

// // its just because raw ptrs are not Sync or Send or Smth (but we dont actually have a raw ptr, we have a PhantomData)
// unsafe impl<Tag: unique_type::Unique, At, Rt> Sync for EventDef<Tag, At, Rt> {}
// unsafe impl<Tag: unique_type::Unique, At, Rt> Send for EventDef<Tag, At, Rt> {}

// impl<Tag: unique_type::Unique, At, Rt> EventDef<Tag, At, Rt> {
//     /// Creates a new event defintion
//     ///
//     /// for a easier (and safe) way of creating an event, see [`event!`]
//     ///
//     /// # Safety
//     ///
//     /// you MUST use `unique_type::new!()` for the type parameter Tag,
//     /// otherwise **THINGS WILL BREAK, INCLUDING YOUR MIND AFTER HOURS OF DEBUGGING**
//     ///
//     /// [`event!`]: crate::event!
//     #[must_use]
//     pub const unsafe fn new(name: &'static str) -> Self {
//         Self {
//             name,
//             _tag: PhantomData,
//             _at: PhantomData,
//             _rt: PhantomData,
//         }
//     }
// }

/// abstraction for registering handlers
#[allow(clippy::module_name_repetitions)]
pub struct EventRegister<S: ?Sized> {
    pub(crate) handlers: Vec<(
        &'static EventDef,
        Box<dyn HandlerCallableErased + Send + Sync + 'static>,
        String,
    )>,
    _stop_t: PhantomData<S>,
}

impl<S: Sync + Send + 'static> EventRegister<S> {
    pub(crate) const fn new() -> Self {
        Self {
            handlers: vec![],
            _stop_t: PhantomData,
        }
    }

    // do not the generic async function pointers
    #[must_use]
    pub fn handler<At, Rt, P>(mut self, def: &'static EventDef, func: P) -> Self
    where
        At: Send + Sync + 'static,
        Rt: Send + Sync + 'static,
        P: for<'a> AsyncFnPtr<'a, S, At, Rt> + Copy + Send + Sync + 'static,
    {
        assert!(def.at == TypeId::of::<At>());
        assert!(def.rt == TypeId::of::<Rt>());
        self.handlers.push((
            def,
            Box::new(HandlerFn::new(func)),
            format!(
                "handler: {}, name: {}, args: {}, return: {}, definition: {:?}",
                type_name::<S>(),
                def.name,
                type_name::<At>(),
                type_name::<Rt>(),
                def,
            ),
        ));
        let _ = def;
        self
    }
}
