#![forbid(unsafe_code)]
pub mod caps;
pub mod convert;
pub mod decode;
pub mod demux;
pub mod device;
pub mod encode;
pub mod link;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation not implemented")]
    NotImplemented,
    #[error("Media backend failed: {0}")]
    Backend(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
