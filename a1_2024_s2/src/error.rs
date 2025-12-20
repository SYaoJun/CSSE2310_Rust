use thiserror::Error;

pub type UQEntropyResult<T, E = UQEntropyError> = Result<T, E>;

#[derive(Debug, Error)]
pub enum UQEntropyError {
    #[error("usage")]
    Usage,
    #[error("file")]
    File(String),
}
