use std::any::Any;

use flume::Sender;
use uuid::Uuid;

use crate::{event::{BusEvent, EventType}, bus::FireEventError, args::EventSpec, util::PossiblyClone};

#[derive(Debug, Clone)]
pub struct BusInterface {
    event_queue: Sender<(BusEvent, EventType, Sender<Result<Option<BusEvent>, FireEventError>>)>,
}

impl BusInterface {
    pub(crate) fn new(sender: Sender<(BusEvent, EventType, Sender<Result<Option<BusEvent>, FireEventError>>)>) -> Self {
        Self {
            event_queue: sender,
        }
    }

    pub async fn fire<S: PossiblyClone + Any + Sync + Send + 'static, A: PossiblyClone + Any + Send + Sync + 'static, R: PossiblyClone + Any + Send + Sync>(&mut self, q: &'static EventSpec<S, A, R>, args: A) -> Result<R, FireEventError> {
        let etype = q.event_variant.clone();
        let args_as_sum_t = (q.convert)(args);

        let raw_event = BusEvent::new(args_as_sum_t, Uuid::new_v4());
        let (response_tx, response_rx) = flume::bounded(1);
        self.event_queue.send((raw_event, etype, response_tx)).unwrap();
        let response = response_rx.into_recv_async().await.unwrap()?;

        match response {
            Some(res) => {
                match res.is_into::<R>() {
                    Ok(expected) => {
                        Ok(*expected)
                    }
                    Err(..) => {
                        Err(FireEventError::InvalidReturnType)
                    }
                }
            }
            None => {
                Ok(q.default_return.as_ref().expect("Send type events must provide a default return").try_clone())
            }
        }
    }

    // pub async fn query<S: BusStop>(&mut self, event: S::Event, args: S::Args) -> S::Response {
    //     // unbounded
    //     debug_assert!(self.event_queue.capacity().is_none());

    //     let (response_tx, response_rx) = flume::bounded(1);

    //     let id = Uuid::new_v4();
    //     let msg = BusEvent::new(event, args, id);

    //     self.event_queue
    //         .send((
    //             msg,
    //             RequestType::Query {
    //                 responder: response_tx,
    //             },
    //         ))
    //         .expect("BusStops must be destroyed before the central handler!");

    //     *response_rx
    //         .recv_async()
    //         .await
    //         .expect("sender sent a response")
    //         .is_into::<ReturnEvent, S::Response>()
    //         .unwrap()
    //         .1
    // }

    // pub async fn send<S: BusStop>(&mut self, event: S::Event, args: S::Args) {
    //     let (notifier_tx, notifier_rx) = flume::bounded(1);

    //     let id = Uuid::new_v4();
    //     let msg = BusEvent::new(event, args, id);

    //     self.event_queue
    //         .send((
    //             msg,
    //             RequestType::Send {
    //                 notifier: notifier_tx,
    //             },
    //         ))
    //         .expect("BusStops must be destroyed before the central handler!");

    //     notifier_rx
    //         .recv_async()
    //         .await
    //         .expect("sender sent a response");
    // }
}
