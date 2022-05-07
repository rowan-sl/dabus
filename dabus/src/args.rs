use std::any::Any;

use crate::{event::EventType, util::PossiblyClone};

// use std::{fmt::Debug, any::{Any, TypeId}};

// use self::sealed::Sealed;

/// please note that when `event_variant` is EventType::Send, the return type `R` is ignored
pub struct EventSpec<S, A: PossiblyClone + Any + Send + Sync + 'static, R: PossiblyClone + Any + Send + Sync > {
    pub event_variant: EventType,
    pub convert: fn(A) -> S,
    // this is what will be returned for Send type events, and MUST be clone if it is not None.
    // it is not required, but recommended, that Send type events have a return type of `R`
    pub default_return: Option<R>,
}

// pub trait IsEventSpec: Sealed {
//     type S;
//     type A;
//     type R;

//     fn variant(&self) -> EventType;
//     fn convert(&self, args: Self::A) -> Self::S;
// }

// impl<S, A, R> Sealed for EventSpec<S, A, R> {}
// impl<S, A, R> IsEventSpec for EventSpec<S, A, R> {
//     type S = S;
//     type A = A;
//     type R = R;
//     fn variant(&self) -> EventType {
//         self.event_variant.clone()
//     }
//     fn convert(&self, args: Self::A) -> Self::S {
//         self.convert(args)
//     }
// }

// mod sealed {
//     pub trait Sealed {}
// }

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

// #[test]
// fn test_decl_event() {
//     #[allow(unused)]
//     enum PossibleArguments {
//         Foo(String),
//         Bar((String, u8))
//     }

//     decl_event!(pub(self), PRINT_N, PossibleArguments, Bar, (String, u8), (), EventType::Query);
// }

