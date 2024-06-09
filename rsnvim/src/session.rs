use rmpv::Value;
use std::os::unix::net::UnixStream;

use crate::{
    client::{Client, Connection},
    error::Error
};

/// The current Neovim session
///
/// Used to send and receive messages to the Neovim session
pub struct Session {
    client: Connection,
}

impl Session {
    /// Create a Neovim connection using a Unix socket
    pub fn from_socket(path: &str) -> Result<Session, Error> {
        let reader = UnixStream::connect(path)?;
        let writer = reader.try_clone()?;
        let mut client = Client::new(reader, writer);
        client.start_event_loop();

        Ok(Session {
            client: Connection::Socket(client),
        })
    }

    /// Synchronous function call
    pub fn call(&mut self, method: &str, args: Vec<Value>) -> Result<(), Error> {
        match self.client {
            Connection::Socket(ref mut client) => client.call(method, args)?,
        };

        Ok(())
    }
}
