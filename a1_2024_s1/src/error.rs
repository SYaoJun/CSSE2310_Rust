use thiserror::Error;

pub type UQUnscrambleResult<T, E = UQUnscrambleError> = Result<T, E>;

#[derive(Debug, Error)]
pub enum UQUnscrambleError {
    #[error("usage")]
    Useage,
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse Int Error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Invalid Expression: {0}")]
    InvalidExpression(String),
}
