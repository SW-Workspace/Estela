#![forbid(unsafe_code)]
pub mod app;
pub mod integration;
mod localization;
pub mod message;
pub mod system;
pub mod theme;
pub mod views;
pub mod widgets;
pub mod windows;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation not implemented")]
    NotImplemented,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
