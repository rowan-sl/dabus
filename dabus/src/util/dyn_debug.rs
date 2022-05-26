use core::fmt::{self, Debug, Formatter};

struct DefaultDbg;

impl Debug for DefaultDbg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("{dyn DynDebug}").finish()
    }
}

pub trait DynDebug {
    fn as_dbg(&self) -> &dyn Debug;
}

impl<T> DynDebug for T {
    default fn as_dbg(&self) -> &dyn Debug {
        &DefaultDbg
    }
}

impl<T: Debug> DynDebug for T {
    fn as_dbg(&self) -> &dyn Debug {
        self
    }
}
