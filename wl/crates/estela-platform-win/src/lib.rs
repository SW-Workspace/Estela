#![cfg(windows)]
pub mod adapter;
pub mod caps;
pub mod capture;
pub mod clock;
pub mod cursor;
pub mod input;
pub mod interop;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation not implemented")]
    NotImplemented,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
