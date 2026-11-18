#![forbid(unsafe_code)]
pub mod clock;
pub mod error;
pub mod input;
pub mod screen;
pub mod session;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation not implemented")]
    NotImplemented,
    #[error("Capture failed: {0}")]
    Backend(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub use input::record_pointer;
pub mod window;
