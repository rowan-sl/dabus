use crate::{stop::BusStopMech, util::GeneralRequirements};

trait BusStopReq: BusStopMech + GeneralRequirements {}
impl<T: BusStopMech + GeneralRequirements> BusStopReq for T {}

pub struct DABus {
    handlers: Vec<Box<dyn BusStopReq + 'static>>,
}

impl DABus {}
