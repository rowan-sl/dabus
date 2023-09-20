
/// declares an EventDef in a global const variable corrisponding to an event with the specified types.
///
/// each event declaration will have a unique type, so you must only use ONE declaration to refer to a event, eg
///
/// ```ignore
/// event!(EVENT_A, (), ());
/// event!(EVENT_B, (), ());
/// ```
/// will NOT have the same type, despite having the same signature
///
/// # Args
/// (
///     name: the name of the const variable produced
///     arg: the type of the arguments for the event
///     ret: the return type of the event
/// )
///
#[macro_export]
macro_rules! event {
    ($name:ident, $arg:ty, $ret:ty) => {
        $crate::__concat_idents::concat_idents!(unique_t = __, $name, _, unique {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            pub struct unique_t;
            unsafe impl $crate::unique_type::Unique for unique_t {}
            pub const $name: &'static $crate::event::EventDef<
                unique_t,
                $arg,
                $ret,
            > = &unsafe {
                $crate::event::EventDef::new(concat!(stringify!($name)))
            };
        });
    };
}
