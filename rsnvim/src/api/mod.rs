mod buffer;
mod common;
mod nvim;
mod tabpage;
mod window;

use crate::error::Error;
use crate::session::Session;

use rmpv::Value;

pub struct Nvim {
    session: Session,
}

impl Nvim {
    pub fn from_session(session: Session) -> Nvim {
        Nvim { session }
    }

    pub fn from_socket(path: &str) -> Result<Nvim, Error> {
        Ok(Nvim {
            session: Session::from_socket(path)?,
        })
    }

    pub fn call(&mut self, method: &str, args: Vec<Value>) {
        self.session.call(method, args).unwrap();
    }
}

pub struct Buffer {}

pub struct Tabpage {}

pub struct Window {}
