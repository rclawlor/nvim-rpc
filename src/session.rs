use std::os::unix::net::UnixStream;

use crate::{
    client::{Client, Connection},
    error::Error
};

/// The current Neovim session
///
/// Used to send and receive messages to the Neovim session
pub struct Session {
    client: Connection
}

impl Session {
    /// Create a Neovim connection using a Unix socket
    pub fn from_socket(path: &str) -> Result<Session, Error> {
        let reader = UnixStream::connect(path)?;
        let writer = reader.try_clone()?;
        let client = Client::new(reader, writer);

        Ok(Session { client: Connection::Socket(client) })
    }
}
