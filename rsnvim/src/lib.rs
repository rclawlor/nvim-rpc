//! # rsnvim
//!
//! `rsnvim` is a crate used to interact with Neovim's API via Rust.
pub mod api;
pub mod client;
pub mod error;
pub mod handler;
pub mod rpc;
pub mod session;

#[cfg(test)]
mod tests {}
