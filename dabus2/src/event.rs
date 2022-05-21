use std::{any::TypeId, fmt::Debug, marker::PhantomData};

use crate::core::dyn_var::DynVar;

use futures::future::{BoxFuture, Future};
pub use unique_type;

/// type for declaring events.
///
/// example:
/// ```rust
/// use dabus2::event::EventDef;
///
/// static TEST_EVENT: &'static EventDef<unique_type::new!(), ()> = &EventDef::new();
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
pub struct Handlers<HSelf> {
    pub(crate) handlers: Vec<Box<dyn RawHandlerErased<HSelf = HSelf>>>,
}

impl<H: Send + 'static> Handlers<H> {
    pub(crate) const fn new() -> Self {
        Self { handlers: vec![] }
    }

    pub async fn handler<
        Tag: unique_type::Unique + Sync + Send + 'static,
        At: Debug + Send + 'static,
        Rt: Debug + Sync + Send + 'static,
        Fr: Future<Output = Rt> + Sync + Send + 'static,
        Ft: for<'a> Fn(&'a mut H, At) -> Fr + Sync + Send + 'static,
    >(
        mut self,
        def: &'static EventDef<Tag, At, Rt>,
        func: Ft,
    ) -> Self {
        self.handlers.push(Box::new(RawHandler::<Tag, H, At, Rt> {
            real_fn: Box::new(move |s, a| Box::pin(func(s, a))),
            _tag: PhantomData,
        }));
        let _ = def;
        self
    }
}

struct RawHandler<Tag: unique_type::Unique, H, At, Rt> {
    pub(crate) real_fn:
        Box<dyn for<'a> Fn(&'a mut H, At) -> BoxFuture<'_, Rt> + Sync + Send + 'static>,
    pub(crate) _tag: PhantomData<Tag>,
}

#[async_trait]
pub trait RawHandlerErased {
    type HSelf;
    unsafe fn releavant_to(&self, tag_id: TypeId) -> bool;
    async unsafe fn call(&self, hself: &mut Self::HSelf, at: DynVar) -> DynVar;
}

#[async_trait]
impl<
        Tag: unique_type::Unique + Sync + Send + 'static,
        H: Send,
        At: Debug + Send + 'static,
        Rt: Debug + Sync + Send + 'static,
    > RawHandlerErased for RawHandler<Tag, H, At, Rt>
{
    type HSelf = H;
    /// # Saftey
    /// tag_id must be the TypeId of the Tag of the event being checked for inequality with
    unsafe fn releavant_to(&self, tag_id: TypeId) -> bool {
        TypeId::of::<Tag>() == tag_id
    }

    async unsafe fn call(&self, hself: &mut Self::HSelf, at: DynVar) -> DynVar {
        let at = at.try_to_unchecked::<At>();
        let res = (self.real_fn)(hself, at);
        DynVar::new::<Rt>(res.await)
    }
}
