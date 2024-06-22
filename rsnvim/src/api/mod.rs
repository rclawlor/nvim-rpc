mod buffer;
mod convert;
mod nvim;
mod tabpage;
mod window;

use crate::error::Error;
use crate::handler::{NotificationHandler, RequestHandler};
use crate::session::Session;

use rmpv::Value;
use std::sync::{Arc, Mutex};

/// The Neovim connection
///
/// This struct exposes each way a user can connect to Neovim's RPC
/// socket, alongside Rust functions for each API method.
pub struct Nvim {
    session: Arc<Mutex<Session>>,
}

impl Nvim {
    pub fn from_session(session: Session) -> Self {
        Nvim { session: Arc::new(Mutex::new(session)) }
    }

    /// Create a Neovim connection using a TCP socket
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
    /// use rsnvim::api::Nvim;
    ///
    /// let mut nvim = match Nvim::from_tcp("127.0.0.1:6666") {
    ///     Ok(nvim) => nvim,
    ///     Err(error) => panic!("Couldn't open TCP socket: {}", error)
    /// };
    /// ```
    pub fn from_tcp(addr: &str) -> Result<Self, Error> {
        Ok(Nvim {
            session: Arc::new(Mutex::new(Session::from_tcp(addr)?)),
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
    pub fn from_parent() -> Result<Self, Error> {
        Ok(Nvim {
            session: Arc::new(Mutex::new(Session::from_parent()?)),
        })
    }

    /// Create a Neovim connection using a Unix socket
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
    /// use rsnvim::api::Nvim;
    ///
    /// let mut nvim = match Nvim::from_unix("/run/user/1000/nvim.XXXXX.X") {
    ///     Ok(nvim) => nvim,
    ///     Err(error) => panic!("Couldn't open UNIX socket: {}", error)
    /// };
    /// ```
    #[cfg(unix)]
    pub fn from_unix(path: &str) -> Result<Self, Error> {
        Ok(Nvim {
            session: Arc::new(Mutex::new(Session::from_unix(path)?)),
        })
    }

    /// Begin the RPC event loop
    ///
    /// This function must be called before RPC messages can be sent as it
    /// handles the return values.
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
        let mut session = self.session.lock().unwrap();
        session
            .start_event_loop(request_handler, notification_handler)
    }

    /// Call a RPC function
    ///
    /// This function allows for arbitrary Neovim function calls though should
    /// not be necessary as the API is exposed within this struct.
    pub fn call(&mut self, method: &str, args: Vec<Value>) {
        let mut session = self.session.lock().unwrap();
        session.call(method, args).unwrap();
    }
}

/// A Neovim buffer
///
/// This struct exposes each way a user can create and interact with a buffer.
#[derive(Clone)]
pub struct Buffer {
    data: Value,
    session: Arc<Mutex<Session>>,
}

impl Buffer {
    pub fn new(data: Value, session: Arc<Mutex<Session>>) -> Self {
        Buffer { data, session }
    }
}

impl From<Buffer> for Value {
    fn from(value: Buffer) -> Self {
        value.data
    }
}

/// A Neovim tabpage
///
/// This struct exposes each way a user can create and interact with a tabpage.
#[derive(Clone)]
pub struct Tabpage {
    data: Value,
    session: Arc<Mutex<Session>>,
}

impl Tabpage {
    pub fn new(data: Value, session: Arc<Mutex<Session>>) -> Self {
        Tabpage { data, session }
    }
}

impl From<Tabpage> for Value {
    fn from(value: Tabpage) -> Self {
        value.data
    }
}

/// A Neovim buffer
///
/// This struct exposes each way a user can create and interact with a window.
#[derive(Clone)]
pub struct Window {
    data: Value,
    session: Arc<Mutex<Session>>,
}

impl Window {
    pub fn new(data: Value, session: Arc<Mutex<Session>>) -> Self {
        Window { data, session }
    }
}

impl From<Window> for Value {
    fn from(value: Window) -> Self {
        value.data
    }
}

