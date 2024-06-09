use rmpv::Value;
use std::io::{BufReader, BufWriter, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

use crate::error::Error;
use crate::rpc;

pub struct Client<R, W>
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
{
    reader: BufReader<R>,
    writer: Arc<Mutex<BufWriter<W>>>,
}

impl<R, W> Client<R, W>
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
{
    pub fn new(reader: R, writer: W) -> Self {
        Client {
            reader: BufReader::new(reader),
            writer: Arc::new(Mutex::new(BufWriter::new(writer))),
        }
    }

    pub fn send_msg(&mut self, method: &str, args: Vec<Value>) -> Result<(), Error> {
        let req = rpc::RpcMessage::RpcRequest {
            msgid: 1,
            method: method.to_owned(),
            params: args,
        };

        let writer = &mut *self.writer.lock().unwrap();
        rpc::encode(writer, req)?;

        Ok(())
    }
}

/// Method of connecting to Neovim session
pub enum Connection {
    /// A Unix socket connection
    Socket(Client<UnixStream, UnixStream>),
}
