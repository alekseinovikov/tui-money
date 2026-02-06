use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("storage error: {0}")]
    Storage(String),
    #[error("record not found")]
    NotFound,
    #[error("invalid data: {0}")]
    InvalidData(String),
}
