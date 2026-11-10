#![forbid(unsafe_code)]
pub mod api;
pub mod decoder_pool;
pub mod export;
pub mod gpu;
pub mod platform;
pub mod preview;
pub mod queues;
pub mod recording;
pub mod state;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation not implemented")]
    NotImplemented,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
