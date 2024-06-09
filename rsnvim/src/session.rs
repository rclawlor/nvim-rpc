use rmpv::Value;
use std::net::TcpStream;
use std::os::unix::net::UnixStream;

use crate::{
    client::{Client, Connection},
    error::Error, handler::{NotificationHandler, RequestHandler}
};

/// The current Neovim session
///
/// Used to send and receive messages to the Neovim session
pub struct Session {
    client: Connection,
}

impl Session {
    /// Create a Neovim connection using a TCP socket
    pub fn from_tcp(addr: &str) -> Result<Session, Error> {
        let reader = TcpStream::connect(addr)?;
        let writer = reader.try_clone()?;
        let client = Client::new(reader, writer);

        Ok(Session {
            client: Connection::TCP(client)
        })
    }

    /// Create a Neovim connection using a Unix socket
    #[cfg(unix)]
    pub fn from_unix(path: &str) -> Result<Session, Error> {
        let reader = UnixStream::connect(path)?;
        let writer = reader.try_clone()?;
        let client = Client::new(reader, writer);

        Ok(Session {
            client: Connection::UNIX(client),
        })
    }

    pub fn start_event_loop(
        &mut self,
        request_handler: Option<Box<dyn RequestHandler + Send>>,
        notification_handler: Option<Box<dyn NotificationHandler + Send>>
    ) {
        match self.client {
            Connection::TCP(ref mut client) => client.start_event_loop(request_handler, notification_handler),
            Connection::UNIX(ref mut client) => client.start_event_loop(request_handler, notification_handler)
        }
    }

    /// Synchronous function call
    pub fn call(&mut self, method: &str, args: Vec<Value>) -> Result<Value, Error> {
        match self.client {
            Connection::TCP(ref mut client) => Ok(client.call(method, args)?),
            Connection::UNIX(ref mut client) => Ok(client.call(method, args)?),
        }
    }
}
