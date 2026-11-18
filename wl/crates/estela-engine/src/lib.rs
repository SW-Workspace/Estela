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
    #[error("Recording failed: {0}")]
    Recording(String),
    #[error(transparent)]
    Capture(#[from] estela_capture::Error),
    #[error(transparent)]
    Format(#[from] estela_format::Error),
    #[error(transparent)]
    Media(#[from] estela_media_ffmpeg::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
