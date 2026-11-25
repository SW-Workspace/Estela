#![cfg(target_os = "linux")]
pub mod adapter;
pub mod caps;
pub mod capture;
pub mod clock;
pub mod cursor;
pub mod encoding;
pub mod input;
pub mod interop;
pub mod microphone;
mod portal_runtime;
pub mod session;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("PipeWire audio failed: {0}")]
    Audio(String),
    #[error("Operation not implemented")]
    NotImplemented,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
pub mod window;
