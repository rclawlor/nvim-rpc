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
    MpscError(String)
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
