use rmpv::Value;

pub trait Remote {
    fn request(&self, name: &str, args: Vec<Value>);
}
