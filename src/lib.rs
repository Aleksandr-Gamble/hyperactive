//! The internet is built upon the http protocol.
//! Such parts of it as are built in Rust, generally use the hyper library, which in turn is built on tokio.
//! This crate is intended to make common hyper tasks a little bit more ergonomic.
//! 

pub mod client;
pub mod server;
pub mod err;
