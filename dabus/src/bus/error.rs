#[derive(Debug, thiserror::Error)]
#[error("Failed to execute event!\n{err:?}")]
pub struct FireEventError {
    err: BaseFireEventError,
}

impl From<BaseFireEventError> for FireEventError {
    fn from(err: BaseFireEventError) -> Self {
        Self { err }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BaseFireEventError {
    #[error("No handler matches the event!")]
    NoHandler,
}
