pub mod client;
pub mod error;
pub mod handler;
pub mod rpc;
pub mod session;


#[cfg(test)]
mod tests {
    use rmpv::Value;

    use super::session::Session;


    #[test]
    fn socket_connection() {
        let mut session = Session::from_socket("/run/user/1000/nvim.45804.0").unwrap();
        session.call("nvim_subscribe", vec![Value::from("Hello")]).unwrap();
    }
}
