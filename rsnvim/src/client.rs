use rmpv::Value;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Stdin, Stdout, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time;

use crate::error::Error;
use crate::handler::{DefaultHandler, NotificationHandler, RequestHandler};
use crate::rpc;

type Sender = mpsc::Sender<Result<Value, Error>>;
type Handles = Arc<Mutex<HashMap<u64, Sender>>>;

/// The client controls the underlying transport mechanism used
/// to communicate with a Neovim instance.
///
/// The `Client` should be instantiated via the `Nvim` or `Session` struct.
pub struct Client<R, W>
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
{
    reader: Option<BufReader<R>>,
    writer: Arc<Mutex<BufWriter<W>>>,
    handles: Handles,
    msg_counter: u64,
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
            msg_counter: 0,
        }
    }

    /// Call a Neovim API method
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
        self.handles.lock().unwrap().insert(msgid, sender);

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
                        return Err(Error::TimeoutError(
                            "Timeout when waiting for RPC response".to_string(),
                        ));
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(Error::MpscError(
                        "Channel disconnected while waiting for RPC response".to_string(),
                    ));
                }
                Ok(val) => return val,
            };
        }
    }

    /// Get the sender responsible for the request with ID `msgid`
    fn find_sender(handles: &Handles, msgid: u64) -> Sender {
        let mut handles = handles.lock().unwrap();

        handles.remove(&msgid).unwrap()
    }

    /// Spawn a thread to handle incoming RPC messages
    pub fn start_event_loop(
        &mut self,
        request_handler: Option<Box<dyn RequestHandler + Send>>,
        notification_handler: Option<Box<dyn NotificationHandler + Send>>,
    ) {
        let r = request_handler.unwrap_or(Box::new(DefaultHandler::new()));
        let n = notification_handler.unwrap_or(Box::new(DefaultHandler::new()));
        Self::dispatch_read_thread(
            self.reader.take().unwrap(),
            self.writer.clone(),
            self.handles.clone(),
            r,
            n,
        );
    }

    /// Spawn new thread to handle reading the underlying Neovim connection
    fn dispatch_read_thread(
        mut reader: BufReader<R>,
        writer: Arc<Mutex<BufWriter<W>>>,
        handles: Handles,
        request_handler: Box<dyn RequestHandler + Send>,
        notification_handler: Box<dyn NotificationHandler + Send>,
    ) -> JoinHandle<()> {
        thread::spawn(move || loop {
            let msg = match rpc::decode(&mut reader) {
                Ok(msg) => msg,
                Err(_) => return,
            };

            match msg {
                rpc::RpcMessage::RpcRequest {
                    msgid,
                    method,
                    params,
                } => {
                    let response = match request_handler.handle_request(msgid, method, params) {
                        Ok(result) => rpc::RpcMessage::RpcResponse {
                            msgid,
                            error: Value::Nil,
                            result,
                        },
                        Err(error) => rpc::RpcMessage::RpcResponse {
                            msgid,
                            error: Value::from(error),
                            result: Value::Nil,
                        },
                    };

                    let writer = &mut *writer.lock().unwrap();
                    rpc::encode(writer, response).unwrap();
                }
                rpc::RpcMessage::RpcResponse {
                    msgid,
                    result,
                    error,
                } => {
                    let sender = Self::find_sender(&handles, msgid);
                    if error != Value::Nil {
                        sender
                            .send(Err(Error::MpscError("Error in RPC response".to_string())))
                            .unwrap();
                    } else {
                        sender.send(Ok(result)).unwrap();
                    }
                }
                rpc::RpcMessage::RpcNotification { method, params } => {
                    notification_handler.handle_notification(method, params)
                }
            };
        })
    }
}

/// Method of connecting to Neovim session
pub enum Connection {
    /// A TCP socket connection
    TCP(Client<TcpStream, TcpStream>),
    /// A stdin/stdout connection
    STDIO(Client<Stdin, Stdout>),
    /// A Unix socket connection
    #[cfg(unix)]
    UNIX(Client<UnixStream, UnixStream>),
}
