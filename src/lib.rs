pub mod client;
pub mod error;
pub mod handler;
pub mod session;


#[cfg(test)]
mod tests {
    use super::session::Session;

    #[test]
    fn socket_connection() {
        let session = Session::from_socket("/run/user/1000/nvim.39287.0").unwrap();
    }
}
