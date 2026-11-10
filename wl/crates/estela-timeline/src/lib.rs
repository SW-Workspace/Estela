#![forbid(unsafe_code)]
pub mod command;
pub mod history;
pub mod hit;
pub mod layout;
pub mod mapping;
pub mod snap;
pub mod timeline;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation not implemented")]
    NotImplemented,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
