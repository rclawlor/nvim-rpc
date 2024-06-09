use rmpv::Value;


pub trait RequestHandler {
    fn handle_request(&self, msgid: u64, method: String, params: Vec<Value>);
}

pub trait NotificationHandler {
    fn handle_notification(&self, method: String, params: Vec<Value>);
}


pub struct DefaultHandler {}

impl DefaultHandler {
    pub fn new() -> Self {
        DefaultHandler {  }
    }
}

impl RequestHandler for DefaultHandler {
    fn handle_request(&self, msgid: u64, method: String, params: Vec<Value>) {
        
    }
}

impl NotificationHandler for DefaultHandler {
    fn handle_notification(&self, method: String, params: Vec<Value>) {
        
    }
}
