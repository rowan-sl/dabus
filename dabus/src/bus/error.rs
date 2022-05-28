use std::any::type_name;

use crate::{EventDef, util::dyn_debug::DynDebug, core::dyn_var::DynVar};

#[derive(Clone, Debug, thiserror::Error)]
#[error("Failed to execute event!\n{err:?}")]
pub struct FireEventError {
    err: BaseFireEventError,
}

impl From<BaseFireEventError> for FireEventError {
    fn from(err: BaseFireEventError) -> Self {
        Self { err }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum BaseFireEventError {
    #[error("No handler matches the event!")]
    NoHandler,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Error while executing bus event:\n{root:#?}")]
pub struct CallTrace {
    pub root: Option<CallEvent>,
}

impl CallTrace {
    pub fn take_root(&mut self) -> Option<CallEvent> {
        self.root.take()
    }

    pub fn set_root(&mut self, root: CallEvent) {
        debug_assert!(self.root.is_none());
        self.root = Some(root);
    }

    /// finds the first failing event in a chain of nested call errors
    pub fn source(&mut self) -> Option<CallEvent> {
        let mut current_root = self.root.clone()?;
        if let Resolution::Success = current_root.resolution.clone()? {
            // no error
            None?
        }
        loop {
            let last_inner = current_root.inner.last()?.to_owned();
            match last_inner.resolution {
                None => None?,// invalid trace
                Some(Resolution::Success) => None?,//no error
                Some(Resolution::NestedCallError) => current_root = last_inner, // more to go
                Some(Resolution::BusError(..)) => break, // we found it!
            }
        }
        Some(current_root)
    }
}


#[derive(Debug, Clone)]
pub enum Resolution {
    Success,
    BusError(FireEventError),
    NestedCallError,
}

#[derive(Debug, Clone)]
pub struct CallEvent {
    pub handler_name: &'static str,
    pub handler_args_t: &'static str,
    pub handler_args: Option<String>,
    pub inner: Vec<Self>,
    pub resolution: Option<Resolution>,
    pub return_t: &'static str,
    pub return_v: Option<String>,
}

#[cfg(not(feature = "backtrace_track_values"))]
impl CallEvent {
    pub fn from_event_def<Tag: unique_type::Unique, At: DynDebug + 'static, Rt: DynDebug + 'static>(def: &'static EventDef<Tag, At, Rt>, _: &At) -> Self {
        Self {
            handler_name: def.name,
            handler_args_t: type_name::<At>(),
            handler_args: None,
            inner: vec![],
            resolution: None,
            return_t: type_name::<Rt>(),
            return_v: None,
        }
    }

    #[inline(always)]
    pub fn set_return(&mut self, _: &DynVar) {}
}

#[cfg(feature = "backtrace_track_values")]
impl CallEvent {
    pub fn from_event_def<Tag: unique_type::Unique, At: DynDebug + 'static, Rt: DynDebug + 'static>(def: &'static EventDef<Tag, At, Rt>, args: &At) -> Self {
        Self {
            handler_name: def.name,
            handler_args_t: type_name::<At>(),
            handler_args: Some(format!("{:#?}", args.as_dbg())),
            inner: vec![],
            resolution: None,
            return_t: type_name::<Rt>(),
            return_v: None,
        }
    }

    pub fn set_return(&mut self, return_v: &DynVar) {
        debug_assert!(self.return_v.is_none());
        let fmt = format!("{:?}", return_v.as_dbg());
        self.return_v = Some(fmt);
    }
}

impl CallEvent {
    pub fn resolve(&mut self, resolution: Resolution) {
        debug_assert!(self.resolution.is_none(), "attempted to set resolution to {:?}, but resolution was already set to: {:?}", resolution, self.resolution);
        self.resolution = Some(resolution);
    }

    pub fn push_inner(&mut self, event: Self) {
        self.inner.push(event);
    }
}
