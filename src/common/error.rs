use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    ConnectionError(String),
    PublishError(String),
    SubscriptionError(String),
    SerializationError(String),
    DeserializationError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            Error::PublishError(msg) => write!(f, "Publish error: {}", msg),
            Error::SubscriptionError(msg) => write!(f, "Subscription error: {}", msg),
            Error::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Error::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
        }
    }
}

impl StdError for Error {}
