/// Error types for nvim-rpc
#[derive(Debug)]
pub enum Error {
    /// An error when instantiating the RPC connection
    ConnectionError(String),
    /// An error when encoding the RPC message
    EncodingError(String)
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::ConnectionError(value.to_string())
    }
}


impl From<rmpv::encode::Error> for Error {
    fn from(value: rmpv::encode::Error) -> Self {
        Self::EncodingError(value.to_string())
    }
}
