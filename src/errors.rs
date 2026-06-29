use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliErrors {
    #[error("unterminated string")]
    UnterminatedString,
}
