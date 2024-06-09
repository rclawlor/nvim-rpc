use rmpv::Value;

pub trait Remote {
    fn call(&self, name: &str, args: Vec<Value>);
}
