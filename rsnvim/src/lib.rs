pub mod api;
pub mod client;
pub mod error;
pub mod handler;
pub mod rpc;
pub mod session;

#[cfg(test)]
mod tests {
    use crate::api::Nvim;

    #[test]
    fn socket_connection() {
        let mut nvim = Nvim::from_socket("/run/user/1000/nvim.47339.0").unwrap();
        nvim.subscribe("Hello".to_string()).unwrap();
    }
}
