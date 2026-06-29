use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliErrors {
    #[error("unterminated string")]
    UnterminatedString,

    #[error("invalid redis value: {0}")]
    InvalidRedisValue(String),

    #[error("invalid redis integer: {0}")]
    InvalidRedisInteger(#[from] ParseIntError),
}
