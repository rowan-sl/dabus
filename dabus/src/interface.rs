use std::any::TypeId;

use flume::Sender;

use crate::{
    bus::error::FireEventError, core::dyn_var::DynVar, util::dyn_debug::DynDebug, EventDef,
};

pub enum BusInterfaceEvent {
    Fire {
        def: TypeId,
        args: DynVar,
        responder: Sender<Result<DynVar, FireEventError>>,
    },
}

#[derive(Debug)]
pub struct BusInterface {
    pub(crate) channel: Sender<BusInterfaceEvent>,
}

impl BusInterface {
    pub(crate) fn new(sender: Sender<BusInterfaceEvent>) -> Self {
        Self { channel: sender }
    }

    /// Fires an event on the bus this event handler is part of
    ///
    /// for more info, see [`DABus::fire`]
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
    ) -> Result<Rt, FireEventError> {
        let _ = def;
        let def = TypeId::of::<Tag>();
        let args = DynVar::new(args);
        let (responder, response) = flume::bounded::<Result<DynVar, FireEventError>>(1);
        self.channel
            .send(BusInterfaceEvent::Fire {
                def,
                args,
                responder,
            })
            .unwrap();
        Ok(response
            .into_recv_async()
            .await
            .unwrap()?
            .try_to::<Rt>()
            .unwrap())
    }

    // unimplemented as this needs to be re-designed
    // /// takes a error (from a nested call, presumablely) and forwards it to the caller of the current event (via the runtime and a deal with the devil)
    // ///
    // /// this is a easy way to handle errors, as it will forward the error, and can produce nice backtraces (soonTM)
    // ///
    // /// this returns ! because as soon as this is polled by the runtime (i think) the future of the bus event will be dropped.
    // /// (hopefully that wont do anything bad?)
    // pub async fn fwd_bus_err(
    //     self, /* not needed, but just to enforce the this-is-the-last-thing-you-do theme */
    //     error: FireEventError,
    // ) -> ! {
    //     self.channel
    //         .send(BusInterfaceEvent::FwdError { error })
    //         .unwrap();
    //     pending!();
    //     unsafe {
    //         // anyone who gets this far deserves it
    //         std::hint::unreachable_unchecked()
    //     }
    // }
}

// /// Utility for handling bus errors inside of bus handlers
// #[async_trait]
// pub trait BusErrorUtil<T> {
//     /// unwraps an `Result`, or forwards the error to [`BusInterface::fwd_bus_err`]
//     async fn unwrap_or_fwd(self, bus: BusInterface) -> T;
// }

// #[async_trait]
// impl<T: Send> BusErrorUtil<T> for Result<T, FireEventError> {
//     async fn unwrap_or_fwd(self, bus: BusInterface) -> T {
//         match self {
//             Ok(x) => x,
//             Err(err) => bus.fwd_bus_err(err).await,
//         }
//     }
// }
