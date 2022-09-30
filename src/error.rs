use thiserror::Error;
use rdkafka::error::KafkaError;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    KafkaError(#[from] KafkaError),
    #[error("{0}")]
    String(String),
}

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Self::String(e.to_string())
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Self::String(e)
    }
}
