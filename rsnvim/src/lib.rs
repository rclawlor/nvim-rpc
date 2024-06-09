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
        let mut nvim = Nvim::from_tcp("127.0.0.1:6666").unwrap();
        nvim.start_event_loop(None, None);
        let namespaces = nvim.get_namespaces().unwrap();
        println!("cargo:warning={}", namespaces);
    }
}
