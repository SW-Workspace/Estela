#![forbid(unsafe_code)]
pub mod audio;
pub mod caps;
pub mod decoder;
pub mod encoder;
pub mod frame;
pub mod interop;
pub mod recording;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Recording backend failed: {0}")]
    Backend(String),
    #[error("Operation not implemented")]
    NotImplemented,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
