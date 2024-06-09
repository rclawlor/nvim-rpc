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
        let mut nvim = Nvim::from_socket("/run/user/1000/nvim.31150.0").unwrap();
        nvim.command("echo \"Hello, Nvim!\"".to_string());
    }
}
