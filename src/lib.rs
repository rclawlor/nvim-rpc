pub mod client;
pub mod error;
pub mod session;


#[cfg(test)]
mod tests {
    use super::session::Session;

    #[test]
    fn socket_connection() {
        let session = Session::from_socket("/run/user/1000/nvim.13257.0").unwrap();
    }
}
