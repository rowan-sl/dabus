use std::{any::TypeId, fmt::Debug, marker::PhantomData};

use crate::core::dyn_var::DynVar;

use futures::future::{Future, BoxFuture};
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
pub struct Handlers<HSelf: 'static> {
    pub(crate) handlers: Vec<(TypeId, Box<dyn HandlerCallable>)>,
    pub(crate) _s: PhantomData<&'static HSelf>,
}

impl<H> Handlers<H> {
    pub(crate) const fn new() -> Self {
        Self { handlers: vec![], _s: PhantomData }
    }

    // do not the generic async function pointers
    pub fn handler<'a, Tag, At, Rt, Fut>(
        mut self,
        def: &'static EventDef<Tag, At, Rt>,
        func: fn(&'a mut H, At) -> Fut,
    ) -> Self
    where
        Tag: unique_type::Unique + Send + Sync + 'static,
        H: Debug + Send + Sync + 'static,
        At: Debug + Send + Sync + 'static,
        Rt: Debug + Send + Sync + 'static,
        Fut: Future<Output = Rt> + Send + Sync + 'static,
    {
        self.handlers.push((
            TypeId::of::<Tag>(),
            Box::new(func),
        ));
        let _ = def;
        self
    }
}

#[async_trait]
pub trait HandlerCallable {
    async unsafe fn call(self, hself: crate::core::dyn_var::UnsafeSendDynVarPtr, at: DynVar) -> DynVar;
}

#[async_trait]
impl<'a, H, At, Rt, Fut> HandlerCallable for fn(&'a mut H, At) -> Fut
where
    H: Debug + Send + Sync + 'static,
    At: Debug + Send + Sync + 'static,
    Rt: Debug + Send + Sync + 'static,
    Fut: Future<Output = Rt> + Send + Sync + 'a,
{
    /// # Saftey
    /// hself and at MUST be the correct types (H and At on the impl block)
    ///
    /// the reference behind hself MUST be valid for 'a (i think) (use crate::core::dyn_var::borrowed_ptr_mut)
    async unsafe fn call(self, hself: crate::core::dyn_var::UnsafeSendDynVarPtr /* since expressing lifetimes like this SUCKS, and raw pointers are not Send or Sync */, at: DynVar) -> DynVar {
        let hself = (*hself.0).as_mut_unchecked::<H>();
        let at = at.try_to_unchecked::<At>();
        let future = self(hself, at);
        let result = future.await;
        DynVar::new(result)
    }
}

// struct RawHandler<'a, 'b: 'a, Tag: unique_type::Unique, H: 'a, At, Rt> {
//     pub(crate) real_fn: Box<dyn Fn(&'a mut H, At) -> BoxFuture<'b, Rt> + Sync + Send + 'b>,
//     pub(crate) _tag: PhantomData<Tag>,
// }

// #[async_trait]
// pub trait RawHandlerErased {
//     type HSelf;
//     unsafe fn releavant_to(&self, tag_id: TypeId) -> bool;
//     async unsafe fn call(&self, hself: &mut Self::HSelf, at: DynVar) -> DynVar;
// }

// #[async_trait]
// impl<
//         'a, 'b, 'c,
//         Tag: unique_type::Unique + Sync + Send + 'static,
//         H: Sync + Send + 'a,
//         At: Debug + Sync + Send,
//         Rt: Debug + Sync + Send,
//     > RawHandlerErased for RawHandler<'a, 'b, Tag, H, At, Rt>
// where
//     'c: 'a,
//     'b: 'a,
// {
//     type HSelf = H;
//     /// # Saftey
//     /// tag_id must be the TypeId of the Tag of the event being checked for inequality with
//     unsafe fn releavant_to(&self, tag_id: TypeId) -> bool {
//         TypeId::of::<Tag>() == tag_id
//     }

//     async unsafe fn call(&'c self, hself: &'c mut Self::HSelf, at: DynVar) -> DynVar
//     {
//         let at = at.try_to_unchecked::<At>();
//         let res = (self.real_fn)(hself, at);
//         DynVar::new::<Rt>(res.await)
//     }
// }


// // no touchie
// //
// // if, like me who wrote it, you touchie, i give you my condolences
// pub trait ToBoxed<'a, 'b: 'a, H: 'a, At, Rt> {
//     fn to_boxed(self) -> Box<dyn Fn(&'a mut H, At) -> BoxFuture<'b, Rt> + 'b>;
// }

// impl<'a, 'b: 'a, H: 'a, At, Rt, F, Fut> ToBoxed<'a, 'b, H, At, Rt> for F
// where
//     F: Fn(&'a mut H, At) -> Fut + 'b,
//     Fut: ::futures::Future<Output = Rt> + Send + Sync + 'b,
// {
//     fn to_boxed(self) -> Box<dyn Fn(&'a mut H, At) -> BoxFuture<'b, Rt> + 'b> {
//         Box::new(move |h, at| Box::pin(self(h, at)))
//     }
// }

// pub trait ToBoxedPtr<'a, 'b: 'a, H: 'static, At: 'static, Rt: 'static> {
//     fn to_boxed(self) -> Box<dyn Fn(&'a mut H, At) -> BoxFuture<'b, Rt> + 'b>;
// }

// impl<'a, 'b: 'a, H: 'static, At: 'static, Rt: 'static, Fut: Future<Output=Rt> + Send + Sync + 'b> ToBoxedPtr<'a, 'b, H, At, Rt> for fn(&'a mut H, At) -> Fut {
//     fn to_boxed(self) -> Box<dyn Fn(&'a mut H, At) -> BoxFuture<'b, Rt> + 'b> {
//         Box::new(move |h, a| Box::pin(self(h, a)))
//     }
// }
