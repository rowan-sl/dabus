pub mod async_fn_ptr;

use std::{any::TypeId, marker::PhantomData};

pub use unique_type;

use self::async_fn_ptr::{HandlerCallableErased, AsyncFnPtr, HandlerFn};

/// type for declaring events.
///
/// example:
/// ```rust
/// use dabus2::event::EventDef;
///
/// static TEST_EVENT: &'static EventDef<unique_type::new!(), ()> = unsafe { &EventDef::new() };
/// ```
pub struct EventDef<
    Tag: unique_type::Unique,
    At,
    Rt = (), /* if At is `()`, than this event is eligeble for lazy evaluation */
> {
    _tag: PhantomData<*const Tag /* dropck */>,
    _at: PhantomData<*const At /* also dropck */>,
    _rt: PhantomData<*const Rt /* also dropck */>,
}

// its just because raw ptrs are not Sync or Send or Smth (but we dont actually have a raw ptr, we have a PhantomData)
unsafe impl<Tag: unique_type::Unique, At, Rt> Sync for EventDef<Tag, At, Rt> {}
unsafe impl<Tag: unique_type::Unique, At, Rt> Send for EventDef<Tag, At, Rt> {}

impl<Tag: unique_type::Unique, At, Rt> EventDef<Tag, At, Rt> {
    /// # Saftey
    /// you MUST use unique_type::new!() for the type parameter Tag,
    /// otherwise **THINGS WILL BREAK, INCLUDING YOUR MIND AFTER HOURS OF DEBUGGING**
    pub const unsafe fn new() -> Self {
        Self {
            _tag: PhantomData,
            _at: PhantomData,
            _rt: PhantomData,
        }
    }
}

/// abstraction for registering handlers
pub struct EventRegister<S> {
    pub(crate) handlers: Vec<(
        TypeId,
        Box<dyn HandlerCallableErased + Send + Sync + 'static>,
    )>,
    _stop_t: PhantomData<S>
}

impl<S: Sync + Send + 'static> EventRegister<S> {
    pub(crate) const fn new() -> Self {
        Self { handlers: vec![], _stop_t: PhantomData }
    }

    // do not the generic async function pointers
    pub fn handler<Tag, At, Rt, P>(
        mut self,
        def: &'static EventDef<Tag, At, Rt>,
        func: P,
    ) -> Self
    where
        Tag: unique_type::Unique + Send + Sync + 'static,
        At: Send + Sync + 'static,
        Rt: Send + Sync + 'static,
        P: for<'a> AsyncFnPtr<'a, S, At, Rt> + Copy + Send + Sync + 'static,
    {
        self.handlers.push((
            TypeId::of::<Tag>(),
            Box::new(HandlerFn::new(func))
        ));
        let _ = def;
        self
    }
}
