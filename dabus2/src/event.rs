use std::{any::TypeId, fmt::Debug, marker::PhantomData, ops::Deref};

use crate::{core::dyn_var::DynVar, stop::BusStop};

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
pub struct EventRegister<E> {
    pub(crate) handlers: Vec<(
        TypeId,
        E,
    )>,
}

impl<E> EventRegister<E> {
    pub(crate) const fn new() -> Self {
        Self { handlers: vec![] }
    }

    // do not the generic async function pointers
    pub fn handler<Tag, At, Rt, Fut>(
        mut self,
        def: &'static EventDef<Tag, At, Rt>,
        func: fn(At) -> E,
    ) -> Self
    where
        Tag: unique_type::Unique + Send + Sync + 'static,
        At: Debug + Send + Sync + 'static,
        Rt: Debug + Send + Sync + 'static,
    {

        let _ = def;
        self
    }
}

// #[async_trait]
// pub trait HandlerCallableLVL2<H: 'static> {
//     unsafe fn call<'c>(&'c self, hself: &'c mut H, at: DynVar) -> BoxFuture<'_, DynVar>;
// }

// impl<T, H: 'static, U> HandlerCallableLVL2<H> for T
// where
//     T: HandlerCallable<H, Unused = U>
// {
//     unsafe fn call<'c>(&'c self, hself: &'c mut H, at: DynVar) -> BoxFuture<'_, DynVar> {
//         HandlerCallable::call(self, hself, at)
//     }
// }

// pub trait HandlerCallableErased<H: 'static> {
//     unsafe fn call_erased(&self, hself: &mut H, at: DynVar) -> BoxFuture<'_, DynVar>;
// }

// impl<T, H: 'static, At, Rt, F: 'static> HandlerCallableErased<H> for Box<dyn HandlerCallable<T, H, At, Rt, F> + Sync + Send>
// where
//     H: Debug + Send + Sync + 'static,
//     At: Debug + Send + Sync + 'static,
//     Rt: Debug + Send + Sync + 'static,
// {
//     unsafe fn call_erased(&self, hself: &mut H, at: DynVar) -> BoxFuture<'_, DynVar> {
//         Box::pin(async {
//             let at = DynVar::try_to_unchecked::<At>(at);
//             let res: Rt = <<Self as Deref>::Target as HandlerCallable<T, H, At, Rt, F>>::call(self, hself, at).await;
//             DynVar::new(res)
//         })
//     }
// }

// pub trait HandlerCallable<T, H: 'static, At, Rt, F> {
//     unsafe fn call<'c>(&'c self, hself: &'c mut H, at: At) -> BoxFuture<'_, Rt>
//     where
//         F: Future<Output = Rt> + Send + Sync + 'c,
//         T: FunctionPointer<'c, H, At, F>;
// }

// // impl<'a, T, H, At, Rt, Fut> HandlerCallable<H, At, Rt> for T
// // where
// //     H: Debug + Send + Sync + 'static,
// //     At: Debug + Send + Sync + 'static,
// //     Rt: Debug + Send + Sync + 'static,
// //     T: FunctionPointer<'a, H, At, Fut>,
// //     Fut: Future<Output = Rt> + Send + Sync + 'a,
// // {
// //     /// # Saftey
// //     /// hself and at MUST be the correct types (H and At on the impl block)
// //     ///
// //     /// the reference behind hself MUST be valid for 'a (i think) (use crate::core::dyn_var::borrowed_ptr_mut)
// //     unsafe fn call(&self, hself: &mut H, at: At) -> BoxFuture<'_, Rt>
// //     {
// //         Box::pin(async {
// //             let future = self(hself, at);
// //             let result = future.await;
// //             result
// //         })
// //         // let at = at.try_to_unchecked::<At>();
// //         // fn scoped<'h2, 'h1: 'h2, H, At, Rt, Ret, F: Fn(&'h1 mut H, At) -> Ret>(
// //         //     f: F,
// //         //     hs: &'h1 mut H,
// //         //     at: At,
// //         // ) -> Ret {
// //         //     f(hs, at)
// //         // }
// //         // scoped::<'a, 'c, H, At, Rt, _, _>(
// //         //     move |hself, at| {
// //         //         Box::pin(async {
// //         //             let future = self(hself, at);
// //         //             let result = future.await;
// //         //             DynVar::new(result)
// //         //         })
// //         //     },
// //         //     hself,
// //         //     at,
// //         // )
// //     }
// // }


// impl<T, H, At, Rt, Fut> HandlerCallable<T, H, At, Rt, Fut> for T
// where
//     T: Send + Sync + 'static,
//     H: Debug + Send + Sync + 'static,
//     At: Debug + Send + Sync + 'static,
//     Rt: Debug + Send + Sync + 'static,
// {
//     /// # Saftey
//     /// hself and at MUST be the correct types (H and At on the impl block)
//     ///
//     /// the reference behind hself MUST be valid for 'a (i think) (use crate::core::dyn_var::borrowed_ptr_mut)
//     unsafe fn call<'c>(self: &'c T, hself: &'c mut H, at: At) -> BoxFuture<'_, Rt>
//     where
//         Fut: Future<Output = Rt> + Send + Sync + 'c,
//         T: FunctionPointer<'c, H, At, Fut>,
//     {
//         Box::pin(async {
//             let future = self(hself, at);
//             let result = future.await;
//             result
//         })
//     }
// }

// trait FunctionPointer<'a, A: 'a, B, C>: Fn(&'a mut A, B) -> C {}
// impl<'a, A: 'a, B, C> FunctionPointer<'a, A, B, C> for fn(&'a mut A, B) -> C {}

#[derive(Debug)]
struct Test {

}

impl Test {
    pub async fn do_thing(&mut self, n: u8) {
        println!("Called with {n}");
    }
}

pub async fn test() {
    let t = Test {};
    let dyn_fn: Box<dyn HandlerCallableErased> = Box::new(HandlerFn::new(Test::do_thing));
    let mut dyn_t = DynVar::new(t);
    unsafe { dyn_fn.call(&mut dyn_t, DynVar::new(10)).await };
    drop(dyn_t);
}

trait AsyncFnPtr<'a, H: 'a, At, Rt> {
    type Fut: Future<Output = Rt> + Send + 'a;
    fn call(self, h: &'a mut H, a: At) -> Self::Fut;
}

impl<'a, H: 'a, At, Fut: Future + Send + 'a, F: FnOnce(&'a mut H, At) -> Fut> AsyncFnPtr<'a, H, At, Fut::Output> for F {
    type Fut = Fut;
    fn call(self, h: &'a mut H, a: At) -> Self::Fut {
        self(h, a)
    }
}

struct HandlerFn<H: 'static, At: 'static, Rt: 'static, P>
where
    P: for<'a> AsyncFnPtr<'a, H, At, Rt> + Copy
{
    f: P,
    _t: PhantomData<&'static (H, At, Rt)>
}

impl<H: 'static + Send, At: 'static + Send, Rt: 'static, P: 'static> HandlerFn<H, At, Rt, P>
where
    P: for<'a> AsyncFnPtr<'a, H, At, Rt> + Send + Copy,
{
    pub fn new(f: P) -> Self {
        Self {
            f,
            _t: PhantomData
        }
    }

    pub fn call<'a, 'b>(&'b self, h: &'a mut H, a: At) -> BoxFuture<'a, Rt> {
        let f = self.f;
        Box::pin(async move {
                f.call(h, a).await
        })
    }
}

trait HandlerCallableErased {
    unsafe fn call<'a>(&'a self, h: &'a mut DynVar, a: DynVar) -> BoxFuture<'a, DynVar>;
}

impl<H, At, Rt, P> HandlerCallableErased for HandlerFn<H, At, Rt, P>
where
    P: for<'a> AsyncFnPtr<'a, H, At, Rt> + Send + Sync + Copy + 'static,
    H: Debug + Send + Sync + 'static,
    At: Debug + Send + Sync + 'static,
    Rt: Debug + Send + Sync + 'static,
{
    unsafe fn call<'a>(&'a self, h: &'a mut DynVar, a: DynVar) -> BoxFuture<'a, DynVar> {
        Box::pin(async move {
            let h = h.as_mut_unchecked::<H>();
            let a = a.try_to_unchecked::<At>();
            let r = self.call(h, a).await;
            DynVar::new(r)
        })
    }
}

struct LifetimeToken<'a, T: 'a>(&'a mut T);

fn limited_lifetime<F>(h: &mut Test, a: u8, f: F) where F: for<'a> AsyncFnPtr<'a, Test, u8, ()> {
    let token = LifetimeToken(h);
    f.call(token.0, a);
    drop(token);
}

fn call_ltd<'s: 'h, 'h, H, At, Rt, Fut>(h: &'s mut H, a: At, ptr: fn(&'h mut H, At) -> Fut) -> BoxFuture<'s, Rt>
where
    H: Sync + Send + 'static,
    At: Sync + Send + 'static,
    Fut: Future<Output = Rt> + Sync + Send + 's,
{
    Box::pin(async move {
        ptr(h, a).await
    })
}
