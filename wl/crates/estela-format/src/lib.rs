#![forbid(unsafe_code)]
pub mod boxes;
pub mod consts;
pub mod extract;
pub mod index;
pub mod migrate;
pub mod reader;
pub mod recover;
pub mod slots;
pub mod tracks;
pub mod writer;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation not implemented")]
    NotImplemented,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
