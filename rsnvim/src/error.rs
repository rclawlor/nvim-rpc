use rmpv::Value;

/// Error types for rsnvim
#[derive(Debug)]
pub enum Error {
    /// An error when instantiating the RPC connection
    ConnectionError(String),
    /// An error when decoding the RPC message
    DecodingError(String),
    /// An error when encoding the RPC message
    EncodingError(String),
    /// An error when the RPC response time is exceeded
    TimeoutError(String),
    /// An error when the MPSC disconnects
    MpscError(String),
    /// An error when an RPC method is not implemented
    NotImplemented(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Error::ConnectionError(err) => err,
                Error::DecodingError(err) => err,
                Error::EncodingError(err) => err,
                Error::TimeoutError(err) => err,
                Error::MpscError(err) => err,
                Error::NotImplemented(err) => err,
            }
        )
    }
}

impl From<Error> for Value {
    fn from(value: Error) -> Self {
        Value::from(format!("{}", value))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::ConnectionError(value.to_string())
    }
}

impl From<rmpv::decode::Error> for Error {
    fn from(value: rmpv::decode::Error) -> Self {
        Self::DecodingError(value.to_string())
    }
}

impl From<rmpv::encode::Error> for Error {
    fn from(value: rmpv::encode::Error) -> Self {
        Self::EncodingError(value.to_string())
    }
}
