#![forbid(unsafe_code)]
pub mod color;
pub mod edit;
pub mod event;
pub mod geometry;
pub mod ids;
pub mod project;
pub mod time;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation not implemented")]
    NotImplemented,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub use {color::*, edit::*, event::*, geometry::*, ids::*, project::*, time::*};
