use rmpv::Value;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Write};
use std::os::unix::net::UnixStream;
use std::thread::{self, JoinHandle};
use std::time;
use std::sync::{Arc, mpsc, Mutex};

use crate::error::Error;
use crate::rpc::{self, decode};

type Sender = mpsc::Sender<Result<Value, Error>>;
type Handles = Arc<Mutex<HashMap<u64, Sender>>>;


pub struct Client<R, W>
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
{
    reader: Option<BufReader<R>>,
    writer: Arc<Mutex<BufWriter<W>>>,
    handles: Handles,
    msg_counter: u64
}

impl<R, W> Client<R, W>
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
{
    pub fn new(reader: R, writer: W) -> Self {
        let handles = Arc::new(Mutex::new(HashMap::new()));
        Client {
            reader: Some(BufReader::new(reader)),
            writer: Arc::new(Mutex::new(BufWriter::new(writer))),
            handles: handles.clone(),
            msg_counter: 0
        }
    }

    pub fn call(&mut self, method: &str, args: Vec<Value>) -> Result<Value, Error> {
        let msgid = self.msg_counter;
        self.msg_counter += 1;

        let req = rpc::RpcMessage::RpcRequest {
            msgid,
            method: method.to_owned(),
            params: args,
        };

        // Keep track of sender to return the response to the correct receiver
        let (sender, receiver) = mpsc::channel();
        self.handles
            .lock()
            .unwrap()
            .insert(msgid, sender);

        let writer = &mut *self.writer.lock().unwrap();
        rpc::encode(writer, req)?;

        let dur = time::Duration::from_secs(1);
        let delay = time::Duration::from_millis(1);
        let instant = time::Instant::now();
        loop {
            match receiver.try_recv() {
                Err(mpsc::TryRecvError::Empty) => {
                    thread::sleep(delay);
                    if instant.elapsed() >= dur {
                        return Err(Error::TimeoutError("Timeout when waiting for RPC response".to_string()));
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(Error::MpscError("Channel disconnected while waiting for RPC response".to_string()));
                }
                Ok(val) => return val,
            };
        }
    }

    fn find_sender(handles: &Handles, msgid: u64) -> Sender {
        let mut handles = handles.lock().unwrap();

        handles.remove(&msgid).unwrap()
    }

    pub fn start_event_loop(&mut self) { 
        Self::dispatch_read_thread(self.reader.take().unwrap(), self.handles.clone());
    }

    fn dispatch_read_thread(mut reader: BufReader<R>, handles: Handles) -> JoinHandle<()> {
        thread::spawn(move || loop {
            let msg = match rpc::decode(&mut reader) {
                Ok(msg) => msg,
                Err(_) => return
            };

            match msg {
                rpc::RpcMessage::RpcRequest {
                    msgid,
                    method,
                    params,
                } => {},
                rpc::RpcMessage::RpcResponse {
                    msgid,
                    result,
                    error,
                } => {
                    let sender = Self::find_sender(&handles, msgid);
                    if error != Value::Nil {
                        sender.send(Err(Error::MpscError("Error in RPC response".to_string()))).unwrap();
                    } else {
                        sender.send(Ok(result)).unwrap();
                    }
                }
                rpc::RpcMessage::RpcNotification { method, params } => {}
            };
        })
    }

    fn read_handler() {

    }
}

/// Method of connecting to Neovim session
pub enum Connection {
    /// A Unix socket connection
    Socket(Client<UnixStream, UnixStream>),
}
