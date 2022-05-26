#[macro_export]
macro_rules! event {
    ($name:ident, $arg:ty, $ret:ty) => {
        pub static $name: &'static $crate::event::EventDef<
            $crate::event::unique_type::new!(),
            $arg,
            $ret,
        > = &unsafe { $crate::event::EventDef::new(concat!(stringify!($name))) };
    };
}
