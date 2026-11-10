#![forbid(unsafe_code)]
pub mod config;
pub mod credentials;
pub mod pool;
pub mod rebuild;
pub mod repo;
pub mod scan;
pub mod server;
pub mod watch;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation not implemented")]
    NotImplemented,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
