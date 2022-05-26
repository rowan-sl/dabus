//! # generic type+lifetime erased asynchrounous function pointers ftw
//!
//! this took me an indescribable ammount of time to figure out
//!
//! ## A warning to travlers
//!
//! no touchie

use crate::{core::dyn_var::DynVar, interface::BusInterface};

use core::marker::PhantomData;

use futures::future::{BoxFuture, Future};

pub trait AsyncFnPtr<'a, H: 'a, At, Rt> {
    type Fut: Future<Output = Rt> + Send + 'a;
    fn call(self, h: &'a mut H, a: At, i: BusInterface) -> Self::Fut;
}

impl<'a, H: 'a, At, Fut: Future + Send + 'a, F: FnOnce(&'a mut H, At, BusInterface) -> Fut>
    AsyncFnPtr<'a, H, At, Fut::Output> for F
{
    type Fut = Fut;
    fn call(self, h: &'a mut H, a: At, i: BusInterface) -> Self::Fut {
        self(h, a, i)
    }
}

#[derive(Clone)]
pub struct HandlerFn<H: 'static, At: 'static, Rt: 'static, P>
where
    P: for<'a> AsyncFnPtr<'a, H, At, Rt> + Copy,
{
    f: P,
    _t: PhantomData<&'static (H, At, Rt)>,
}

impl<H: 'static + Send, At: 'static + Send, Rt: 'static, P: 'static> HandlerFn<H, At, Rt, P>
where
    P: for<'a> AsyncFnPtr<'a, H, At, Rt> + Send + Copy,
{
    pub fn new(f: P) -> Self {
        Self { f, _t: PhantomData }
    }

    pub fn call<'a, 'b>(&'b self, h: &'a mut H, a: At, i: BusInterface) -> BoxFuture<'a, Rt> {
        let f = self.f;
        Box::pin(async move { f.call(h, a, i).await })
    }
}

pub trait HandlerCallableErased {
    unsafe fn call<'a>(
        &'a self,
        h: &'a mut DynVar,
        a: DynVar,
        i: BusInterface,
    ) -> BoxFuture<'a, DynVar>;
}

impl<H, At, Rt, P> HandlerCallableErased for HandlerFn<H, At, Rt, P>
where
    P: for<'a> AsyncFnPtr<'a, H, At, Rt> + Send + Sync + Copy + 'static,
    H: Send + Sync + 'static,
    At: Send + Sync + 'static,
    Rt: Send + Sync + 'static,
{
    unsafe fn call<'a>(
        &'a self,
        h: &'a mut DynVar,
        a: DynVar,
        i: BusInterface,
    ) -> BoxFuture<'a, DynVar> {
        Box::pin(async move {
            let h = h.as_mut_unchecked::<H>();
            let a = a.try_to_unchecked::<At>();
            let r = self.call(h, a, i).await;
            DynVar::new(r)
        })
    }
}
