use rmpv::Value;
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

use crate::{
    client::{Client, Connection},
    error::Error,
    handler::{NotificationHandler, RequestHandler},
};

/// The current Neovim session
///
/// Used to send and receive messages to the Neovim session
pub struct Session {
    client: Connection,
}

impl Session {
    /// Create a session using a TCP socket
    ///
    /// This allows RPC communication with a Neovim instance started with
    /// ```shell
    /// nvim --listen 127.0.0.1:6666
    /// ```
    /// The current Neovim server can be found using
    /// ```shell
    /// :echo v:servername
    /// ```
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rsnvim::session::Session;
    ///
    /// let mut session = match Session::from_tcp("127.0.0.1:6666") {
    ///     Ok(session) => session,
    ///     Err(error) => panic!("Couldn't open TCP socket: {}", error)
    /// };
    /// ```
    pub fn from_tcp(addr: &str) -> Result<Session, Error> {
        let reader = TcpStream::connect(addr)?;
        let writer = reader.try_clone()?;
        let client = Client::new(reader, writer);

        Ok(Session {
            client: Connection::TCP(client),
        })
    }

    /// Create a Neovim connection using stdin/stdout
    ///
    /// This allows RPC communication with the Neovim instance that spawned
    /// this process.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rsnvim::api::Nvim;
    ///
    /// let mut nvim = match Nvim::from_parent() {
    ///     Ok(nvim) => nvim,
    ///     Err(error) => panic!("Couldn't connect to parent session: {}", error)
    /// };
    /// ```
    pub fn from_parent() -> Result<Session, Error> {
        let client = Client::new(std::io::stdin(), std::io::stdout());

        Ok(Session {
            client: Connection::STDIO(client)
        })
    }


    /// Create a session using a Unix socket
    ///
    /// This allows RPC communication with any Neovim instance as it
    /// creates a default RPC socket on startup.
    ///
    /// The current Neovim server can be found using
    /// ```shell
    /// :echo v:servername
    /// ```
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rsnvim::session::Session;
    ///
    /// let mut session = match Session::from_unix("/run/user/1000/nvim.XXXXX.X") {
    ///     Ok(session) => session,
    ///     Err(error) => panic!("Couldn't open UNIX socket: {}", error)
    /// };
    /// ```
    #[cfg(unix)]
    pub fn from_unix(path: &str) -> Result<Session, Error> {
        let reader = UnixStream::connect(path)?;
        let writer = reader.try_clone()?;
        let client = Client::new(reader, writer);

        Ok(Session {
            client: Connection::UNIX(client),
        })
    }

    /// Begin the RPC event loop
    ///
    /// This function must be called before RPC messages can be sent as it
    /// handles the return values, though it is also exposed though the `Nvim`
    /// struct.
    ///
    /// This function allows for up to two custom handlers:
    ///
    /// # request_handler
    /// The `request_handler` struct must implement the `RequestHandler` trait
    /// which then allows it to process incoming RPC requests from Neovim. If
    /// `None` is passed the `DefaultHandler` will be used which responds with
    /// a `NotImplemented` error.
    ///
    /// # notification_handler
    /// The `notification_handler` struct must implement the 'NotificationHandler'
    /// trait which then allows it to process incoming RPC notifications from Neovim.
    /// If 'None' is passed the `DefaultHandler` will be used which ignores all
    /// RPC notifications.
    pub fn start_event_loop(
        &mut self,
        request_handler: Option<Box<dyn RequestHandler + Send>>,
        notification_handler: Option<Box<dyn NotificationHandler + Send>>,
    ) {
        match self.client {
            Connection::TCP(ref mut client) => {
                client.start_event_loop(request_handler, notification_handler)
            }
            Connection::STDIO(ref mut client) => {
                client.start_event_loop(request_handler, notification_handler)
            }
            #[cfg(unix)]
            Connection::UNIX(ref mut client) => {
                client.start_event_loop(request_handler, notification_handler)
            }
        }
    }

    /// Call a RPC function
    ///
    /// This function allows for arbitrary Neovim function calls
    pub fn call(&mut self, method: &str, args: Vec<Value>) -> Result<Value, Error> {
        match self.client {
            Connection::TCP(ref mut client) => Ok(client.call(method, args)?),
            Connection::STDIO(ref mut client) => Ok(client.call(method, args)?),
            #[cfg(unix)]
            Connection::UNIX(ref mut client) => Ok(client.call(method, args)?),
        }
    }
}
