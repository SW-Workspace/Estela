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

#[cfg(target_os = "linux")]
pub use estela_platform_linux::encoding::recording_backends;

#[cfg(windows)]
pub use estela_platform_win::encoding::recording_backends;

#[cfg(windows)]
pub use estela_platform_win::encoding::native_factory;

#[cfg(target_os = "linux")]
pub fn native_factory(
    _backend: &estela_media::recording::RecordingBackend,
) -> Option<estela_media::recording::RecordingEncoderFactory> {
    None
}

pub(crate) fn setup_graphical_subprocesses() {
    #[cfg(windows)]
    estela_media_ffmpeg::set_command_setup(estela_platform_win::window::hide_console);
}
