use rmpv::Value;

use crate::error::Error;

/// A trait for handling incoming RPC requests
///
/// Implementing this trait allows users to define custom RPC methods that can be
/// called from Neovim (e.g. as part a custom user command).
pub trait RequestHandler {
    fn handle_request(
        &self,
        _msgid: u64,
        method: String,
        _params: Vec<Value>,
    ) -> Result<Value, Error> {
        Err(Error::NotImplemented(method))
    }
}

/// A trait for handling incoming RPC notifications
///
/// Implementing this trait allows users to respond to notifications that are sent
/// from Neovim (e.g. as part of a custom user command).
pub trait NotificationHandler {
    fn handle_notification(&self, _method: String, _params: Vec<Value>) {}
}

#[derive(Default)]
pub struct DefaultHandler {}

impl DefaultHandler {
    pub fn new() -> Self {
        DefaultHandler {}
    }
}

impl RequestHandler for DefaultHandler {}

impl NotificationHandler for DefaultHandler {}
