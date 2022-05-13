use crate::{event::EventType, util::GeneralRequirements};

/// Specification for bus events, this is the first argument that is passed to the `bus.fire()` method
///
/// this should only be created through the [`decl_event`] macro
///
///  please note that when `event_variant` is `EventType::Send`, the return type `R` is ignored
pub struct EventSpec<
    S: Send + 'static,
    A: Send + Sync + 'static,
    R: GeneralRequirements + Send + Sync,
> {
    pub event_variant: EventType,
    pub convert: fn(A) -> S,
    // this is what will be returned for Send type events, and MUST be clone if it is not None.
    // it is not required, but recommended, that Send type events have a return type of `R`
    pub default_return: Option<R>,
}

/// Creates a [`EventSpec`] that represents one form of bus event.
/// arguments are as follows (in order)
///
/// vis: visibility specifier for the produced constant. (`pub`, `pub(crate)`)
///     - neat trick: if you want it to be private, since you have to input an argument, use `pub(self)`
///
/// name: the name of the produced constant (const $name)
///
/// event_sum_t: the enum of all events (Event type of the BusStop)
///
/// convert: the variant of event_sum_t that this event is (must be a tuple variant, with exactly one arg)
///
/// arg_t: the type of the argument given to the enum variant
///
/// ret_t: the type returned by thsi event
///
/// def_ret: default return type (returend from send events), set to None for query events, and Some(type) for send events
///
/// event_variant: EventType::Send or Query
///
#[macro_export]
macro_rules! decl_event {
    ($vis:vis, $name:ident, $event_sum_t:ty, $convert:ident, $arg_t:ty, $ret_t:ty, $def_ret:expr, $event_variant:expr) => {
        $vis const $name: &'static $crate::args::EventSpec<$event_sum_t, $arg_t, $ret_t> = &$crate::args::EventSpec {
            event_variant: $event_variant,
            convert: <$event_sum_t>::$convert,
            default_return: $def_ret,
        };
    };
}
