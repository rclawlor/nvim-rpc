mod buffer;
mod common;
mod nvim;
mod tabpage;
mod window;

use crate::error::Error;
use crate::handler::{NotificationHandler, RequestHandler};
use crate::session::Session;

use rmpv::Value;

pub struct Nvim {
    session: Session,
}

impl Nvim {
    pub fn from_session(session: Session) -> Self {
        Nvim { session }
    }

    /// Create a Neovim connection using a TCP socket
    pub fn from_tcp(addr: &str) -> Result<Self, Error> {
        Ok(Nvim {
            session: Session::from_tcp(addr)?,
        })
    }

    /// Create a Neovim connection using a Unix socket
    #[cfg(unix)]
    pub fn from_unix(path: &str) -> Result<Self, Error> {
        Ok(Nvim {
            session: Session::from_unix(path)?,
        })
    }

    pub fn start_event_loop(
        &mut self,
        request_handler: Option<Box<dyn RequestHandler + Send>>,
        notification_handler: Option<Box<dyn NotificationHandler + Send>>
    ) {
        self.session.start_event_loop(request_handler, notification_handler)
    }

    pub fn call(&mut self, method: &str, args: Vec<Value>) {
        self.session.call(method, args).unwrap();
    }
}

pub struct Buffer {}

pub struct Tabpage {}

pub struct Window {}
