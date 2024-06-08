use std::io::{BufReader, BufWriter, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};


pub struct Client<R, W>
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
{
    reader: BufReader<R>,
    writer: Arc<Mutex<BufWriter<W>>>
}

impl<R, W> Client<R, W>
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
{
    pub fn new(reader: R, writer: W) -> Self {
        Client {
            reader: BufReader::new(reader),
            writer: Arc::new(Mutex::new(BufWriter::new(writer)))
        }
    }
}


pub enum Connection {
    Socket(Client<UnixStream, UnixStream>)
}

