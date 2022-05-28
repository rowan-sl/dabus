use std::any::TypeId;

use flume::Sender;

use crate::{
    bus::error::{CallEvent, CallTrace}, core::dyn_var::DynVar, util::dyn_debug::DynDebug, EventDef,
};

#[derive(Debug)]
pub enum BusInterfaceEvent {
    Fire {
        def: TypeId,
        args: DynVar,
        responder: Sender<Result<DynVar, CallTrace>>,
        trace_data: CallEvent,
    },
    FwdBusError {
        error: CallTrace,
        blocker: Sender<()>,
    },
}

/// Provides a limited [`DABus`] like api for handler implementations.
///
/// This is passed to handlers, giving them a way of running actions on the bus that they are being run from.
///
/// [`DABus`]: crate::bus::DABus
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct BusInterface {
    pub(crate) channel: Sender<BusInterfaceEvent>,
}

impl BusInterface {
    pub(crate) const fn new(sender: Sender<BusInterfaceEvent>) -> Self {
        Self { channel: sender }
    }

    /// Fires an event on the bus, running appropreate handlers and returning the result.
    /// This function is similar to [`DABus::fire`] in the sense that from the outside, it behaves the same
    /// however internally it does not. see the `Notes` section for more details
    ///
    /// # Returns
    ///
    /// on success, this returns the return value sent by the handler, as well as a call trace (this will change)
    ///
    /// on failure, this returns only the call trace, which can be used to find what went wrong
    ///
    /// # Panics
    ///
    /// if a handler that is called panics (or the runtime is broken)
    ///
    /// # Errors
    ///
    /// if there is some (expected) error with the runtime. currently this only includes not finding an appropreate handler
    ///
    /// # Notes
    /// like all functions on this struct, this does not execute an event iself but rather forwards it to the current runtime.
    /// this means that if useing this after the scope of the handler it was given to has ended should be considered **Undefined Behavior** (eventually there will be some safeguard to fix this)
    ///
    /// [`DABus::fire`]: crate::bus::DABus::fire
    pub async fn fire<
        Tag: unique_type::Unique,
        At: DynDebug + Sync + Send + 'static,
        Rt: DynDebug + Sync + Send + 'static,
    >(
        &mut self,
        def: &'static EventDef<Tag, At, Rt>,
        args: At,
    ) -> Result<Rt, CallTrace> {
        let trace_data = CallEvent::from_event_def(def, &args);
        let _ = def;
        let def = TypeId::of::<Tag>();
        let args = DynVar::new(args);
        let (responder, response) = flume::bounded::<Result<DynVar, CallTrace>>(1);
        self.channel
            .send(BusInterfaceEvent::Fire {
                def,
                args,
                responder,
                trace_data,
            })
            .unwrap();
        Ok(response
            .into_recv_async()
            .await
            .unwrap()?
            .try_to::<Rt>()
            .unwrap())
    }

    /// takes a error (from a nested call, presumablely) and forwards it to the caller of the current event (via the runtime and a deal with the devil)
    ///
    /// this is a easy way to handle errors, as it will forward the error, and can produce nice backtraces
    ///
    /// # Panics
    ///
    /// it shouldent, unless something is horribly wrong with the library
    ///
    /// # Footguns
    ///
    /// - this function (from the perspective of the handler) will never return, but from the persepective of the program it will, so keep that in mind.
    ///
    /// - see the `Notes` section in [`BusInterface::fire`]
    pub async fn fwd_bus_err(
        &self,
        error: CallTrace,
    ) -> ! {
        let (blocker, blocks) = flume::bounded::<()>(1);
        self.channel
            .send(BusInterfaceEvent::FwdBusError { error, blocker })
            .unwrap();
        blocks.recv_async().await.unwrap();
        unreachable!()
    }
}

/// Utility for handling bus errors inside of bus handlers
#[async_trait]
pub trait BusErrorUtil<T> {
    /// unwraps an `Result`, or forwards the error to [`BusInterface::fwd_bus_err`].
    ///
    /// see [`BusInterface::fwd_bus_err`] for more information
    async fn unwrap_or_fwd(self, bus: &BusInterface) -> T;
}

#[async_trait]
impl<T: Send> BusErrorUtil<T> for Result<T, CallTrace> {
    /// unwraps an `Result`, or forwards the error to [`BusInterface::fwd_bus_err`].
    ///
    /// see [`BusInterface::fwd_bus_err`] for more information
    async fn unwrap_or_fwd(self, bus: &BusInterface) -> T {
        match self {
            Ok(x) => x,
            Err(err) => bus.fwd_bus_err(err).await,
        }
    }
}
