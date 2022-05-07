use crate::{event::EventType, util::GeneralRequirements};

/// please note that when `event_variant` is EventType::Send, the return type `R` is ignored
pub struct EventSpec<S: Send + 'static, A: Send + Sync + 'static, R: GeneralRequirements + Send + Sync > {
    pub event_variant: EventType,
    pub convert: fn(A) -> S,
    // this is what will be returned for Send type events, and MUST be clone if it is not None.
    // it is not required, but recommended, that Send type events have a return type of `R`
    pub default_return: Option<R>,
}

//TODO docs
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
