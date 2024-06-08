/// Error types for nvim-rpc
#[derive(Debug)]
pub enum Error {
    /// An error when instantiating the RPC connection
    Connection(String)
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Connection(value.to_string())
    }
}
