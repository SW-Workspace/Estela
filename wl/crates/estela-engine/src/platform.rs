#[cfg(target_os = "linux")]
pub use estela_platform_linux::adapter::{exclude_window, listen_shortcuts, prepare, sources};

#[cfg(windows)]
pub use estela_platform_win::adapter::{exclude_window, listen_shortcuts, prepare, sources};

#[cfg(target_os = "linux")]
pub use estela_platform_linux::window::{
    configure as configure_window, presentation_backend, release as release_window,
};

#[cfg(windows)]
pub use estela_platform_win::window::{
    configure as configure_window, presentation_backend, release as release_window,
};
