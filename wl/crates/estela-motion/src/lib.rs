#![forbid(unsafe_code)]
pub mod autozoom;
pub mod camera;
pub mod cursor_path;
pub mod easing;
pub mod spring;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation not implemented")]
    NotImplemented,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
