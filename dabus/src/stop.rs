use std::{any::TypeId, fmt::Debug};

use crate::{
    core::dyn_var::DynVar, event::EventRegister, interface::BusInterface, util::{GeneralRequirements, dyn_debug::DynDebug},
};

pub trait BusStop {
    fn registered_handlers(h: EventRegister<Self>) -> EventRegister<Self>;
}

mod seal {
    pub trait Sealed {}
}

#[async_trait]
pub(crate) trait BusStopMech: Sized + seal::Sealed {
    async unsafe fn handle_raw_event(
        self,
        event_tag_id: TypeId,
        event: DynVar,
        interface: BusInterface,
    ) -> (Self, DynVar);
    fn relevant(&self, event_tag_id: TypeId) -> bool;
}

impl<T> seal::Sealed for T where T: BusStop + Debug + Sized + Send + Sync + 'static {}

#[async_trait]
impl<T> BusStopMech for T
where
    T: BusStop + Debug + Sized + Send + Sync + 'static,
{
    async unsafe fn handle_raw_event(
        mut self,
        event_tag_id: TypeId,
        event: DynVar, /* must be the hidden event type */
        interface: BusInterface,
    ) -> (Self, DynVar) /* the hidden return type */ {
        // TODO make this not query handlers each and every event
        let mut handlers = T::registered_handlers(EventRegister::new())
            .handlers
            .into_iter()
            .filter(|rh| rh.0 == event_tag_id)
            .collect::<Vec<_>>();
        debug_assert!(handlers.len() == 1);
        let handler = handlers.remove(0);

        let mut dyn_self = DynVar::new(self);

        let fut = handler.1.call(&mut dyn_self, event, interface);
        let res = fut.await;

        let typed_self = dyn_self.try_to_unchecked::<Self>();

        (typed_self, res)
    }

    fn relevant(&self, event_tag_id: TypeId) -> bool {
        // TODO make this not query handlers each and every event
        let handlers = T::registered_handlers(EventRegister::new())
            .handlers
            .into_iter()
            .filter(|rh| rh.0 == event_tag_id)
            .collect::<Vec<_>>();
        debug_assert!(handlers.len() <= 1);
        !handlers.is_empty()
    }
}

// this probably can be combined with BusStopMech's behavior to simplify things
pub(crate) struct BusStopMechContainer<B: BusStopMech + GeneralRequirements + Send + Sync + 'static> {
    inner: Option<B>,
}

impl<B: BusStopMech + GeneralRequirements + Send + Sync + 'static> BusStopMechContainer<B> {
    pub const fn new(inner: B) -> Self {
        Self { inner: Some(inner) }
    }

    pub async unsafe fn handle_raw_event(
        &mut self,
        event_tag_id: TypeId,
        event: DynVar, /* must be the hidden event type */
        interface: BusInterface,
    ) -> DynVar {
        let moved_self = self.inner.take().unwrap();
        let (moved_self, res) = moved_self
            .handle_raw_event(event_tag_id, event, interface)
            .await;
        self.inner = Some(moved_self);
        res
    }

    pub fn relevant(&mut self, event_tag_id: TypeId) -> bool {
        self.inner.as_mut().unwrap().relevant(event_tag_id)
    }

    pub fn debug(&self) -> &dyn Debug {
        self.inner.as_dbg()
    }
}

impl<B: BusStopMech + GeneralRequirements + Send + Sync + 'static> seal::Sealed for BusStopMechContainer<B> {}

#[async_trait]
#[doc(hidden)]
pub(crate) trait DynBusStopContainer: seal::Sealed {
    async unsafe fn handle_raw_event(
        &mut self,
        event_tag_id: TypeId,
        event: DynVar, /* must be the hidden event type */
        interface: BusInterface,
    ) -> DynVar;
    fn relevant(&mut self, event_tag_id: TypeId) -> bool;
    fn debug(&self) -> &dyn Debug;
}

#[async_trait]
impl<B: BusStopMech + GeneralRequirements + Send + Sync + 'static> DynBusStopContainer
    for BusStopMechContainer<B>
{
    async unsafe fn handle_raw_event(
        &mut self,
        event_tag_id: TypeId,
        event: DynVar, /* must be the hidden event type */
        interface: BusInterface,
    ) -> DynVar {
        BusStopMechContainer::handle_raw_event(self, event_tag_id, event, interface).await
    }

    fn relevant(&mut self, event_tag_id: TypeId) -> bool {
        BusStopMechContainer::relevant(self, event_tag_id)
    }

    fn debug(&self) -> &dyn Debug {
        self.debug()
    }
}

pub(crate) trait BusStopReq: DynBusStopContainer + GeneralRequirements {}
impl<T: DynBusStopContainer + GeneralRequirements> BusStopReq for T {}

pub(crate) struct BusStopContainer {
    pub(crate) inner: Box<dyn BusStopReq + Send + Sync + 'static>,
}

impl BusStopContainer {
    pub const fn new(inner: Box<dyn BusStopReq + Send + Sync + 'static>) -> Self {
        Self { inner }
    }

    pub async unsafe fn handle_raw_event(
        mut self,
        event_tag_id: TypeId,
        event: DynVar, /* must be the hidden event type */
        interface: BusInterface,
    ) -> (Self, DynVar) {
        let ret = self
            .inner
            .handle_raw_event(event_tag_id, event, interface)
            .await;
        (self, ret)
    }

    pub fn relevant(&mut self, event_tag_id: TypeId) -> bool {
        self.inner.relevant(event_tag_id)
    }

    pub fn debug(&self) -> &dyn Debug {
        self.inner.debug()
    }
}

impl Debug for BusStopContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BusStopContainer")
            .field("inner", self.debug())
            .finish()
    }
}
