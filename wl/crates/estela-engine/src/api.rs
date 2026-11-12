pub use estela_capture::session::{Frame, Shortcut, Source, SourceKind, SourceSelection};
pub use estela_format::tracks::{Recording, ZoomTrigger};

pub fn sources() -> crate::Result<Vec<Source>> {
    Ok(crate::platform::sources()?)
}

pub fn open(path: &std::path::Path) -> crate::Result<Recording> {
    Ok(estela_format::reader::open(path)?)
}

pub fn exclude_window(id: u64) -> bool {
    crate::platform::exclude_window(id)
}

pub fn shortcuts(descriptions: [String; 2]) -> std::sync::mpsc::Receiver<Shortcut> {
    let (sender, receiver) = std::sync::mpsc::channel();
    crate::platform::listen_shortcuts(sender, descriptions);
    receiver
}

pub use estela_capture::window::{InputArea, WindowInput};
pub use raw_window_handle::{DisplayHandle, WindowHandle};

pub fn configure_window(
    window: WindowHandle<'_>,
    display: DisplayHandle<'_>,
    input: &WindowInput,
    scale: f32,
) -> crate::Result<bool> {
    Ok(crate::platform::configure_window(
        window, display, input, scale,
    )?)
}

pub fn release_window(display: DisplayHandle<'_>) {
    crate::platform::release_window(display);
}

pub fn presentation_backend() -> &'static str {
    crate::platform::presentation_backend()
}
