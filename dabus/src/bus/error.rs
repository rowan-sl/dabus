use std::any::type_name;

use crate::{EventDef, util::dyn_debug::DynDebug};

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

#[derive(Debug, Clone)]
pub enum Resolution {
    Success,
    BusError(FireEventError),
}

#[derive(Debug, Clone)]
pub struct CallEvent {
    pub handler_name: &'static str,
    pub handler_args_t: &'static str,
    pub handler_args: String,
    pub inner: Vec<Self>,
    pub resolution: Option<Resolution>,
    pub return_t: &'static str,
    pub return_v: Option<String>,
}

impl CallEvent {
    pub fn from_event_def<Tag: unique_type::Unique, At: DynDebug + 'static, Rt: DynDebug + 'static>(def: &'static EventDef<Tag, At, Rt>, args: &At) -> Self {
        Self {
            handler_name: def.name,
            handler_args_t: type_name::<At>(),
            handler_args: format!("{:#?}", args.as_dbg()),
            inner: vec![],
            resolution: None,
            return_t: type_name::<Rt>(),
            return_v: None,
        }
    }
}
