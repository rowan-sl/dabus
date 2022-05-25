
#[derive(Debug, thiserror::Error)]
#[error("Failed to execute event!")]
pub struct FireEventError {
    err: BaseFireEventError
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
