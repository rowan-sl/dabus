use std::any::Any;

use flume::Sender;
use uuid::Uuid;
use futures::pending;

use crate::{event::{BusEvent, EventType}, bus::FireEventError, args::EventSpec, util::PossiblyClone};

pub enum InterfaceEvent {
    Call(BusEvent, EventType, Sender<Result<Option<BusEvent>, FireEventError>>),
    FwdErr(FireEventError),
}

#[derive(Debug, Clone)]
pub struct BusInterface {
    event_queue: Sender<InterfaceEvent>,
}

impl BusInterface {
    pub(crate) fn new(sender: Sender<InterfaceEvent>) -> Self {
        Self {
            event_queue: sender,
        }
    }

    pub async fn fire<S: PossiblyClone + Any + Sync + Send + 'static, A: PossiblyClone + Any + Send + Sync + 'static, R: PossiblyClone + Any + Send + Sync>(&mut self, q: &'static EventSpec<S, A, R>, args: A) -> Result<R, FireEventError> {
        let etype = q.event_variant.clone();
        let args_as_sum_t = (q.convert)(args);

        let raw_event = BusEvent::new(args_as_sum_t, Uuid::new_v4());
        let (response_tx, response_rx) = flume::bounded(1);
        self.event_queue.send(InterfaceEvent::Call(raw_event, etype, response_tx)).unwrap();
        let response = response_rx.into_recv_async().await.unwrap()?;

        match response {
            Some(res) => {
                match res.is_into::<R>() {
                    Ok(expected) => {
                        Ok(*expected)
                    }
                    Err(actual) => {
                        let expected = std::any::type_name::<Box<R>>();
                        let found = (*actual.into_raw().0).type_name();
                        Err(FireEventError::InvalidReturnType(expected, found))
                    }
                }
            }
            None => {
                Ok(q.default_return.as_ref().expect("Send type events must provide a default return").try_clone())
            }
        }
    }

    /// takes a error (from a nested call, presumablely) and forwards it to the caller of the current event (via the runtime and a deal with the devil)
    ///
    /// this is a easy way to handle errors, as it will forward the error, and can produce nice backtraces (soonTM)
    ///
    /// this returns ! because as soon as this is polled by the runtime (i think) the future of the bus event will be dropped.
    /// (hopefully that wont do anything bad?)
    pub async fn fwd_bus_err(self/* not needed, but just to enforce the this-is-the-last-thing-you-do theme */, error: FireEventError) -> ! {
        self.event_queue.send(InterfaceEvent::FwdErr(error)).unwrap();
        // here is the deal with the devil
        // as long as the thing polling this function is actually the bus call system, this will never get past this point
        pending!();
        unreachable!("For the love of god do not use nested async executors");
    }
}

#[async_trait]
pub trait BusErrorUtil<T> {
    async fn unwrap_or_fwd(self, bus: BusInterface) -> T;
}

#[async_trait]
impl<T: Send> BusErrorUtil<T> for Result<T, FireEventError> {
    async fn unwrap_or_fwd(self, bus: BusInterface) -> T {
        match self {
            Ok(x) => x,
            Err(err) => {
                bus.fwd_bus_err(err).await
            }
        }
    }
}
