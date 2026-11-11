use estela_capture::session::{Session, Source, SourceKind};
use estela_capture::Result;

pub fn sources() -> Result<Vec<Source>> {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return Ok(vec![
            Source {
                id: 0,
                name: String::new(),
                kind: SourceKind::Display,
                selection: estela_capture::session::SourceSelection::SystemPicker,
            },
            Source {
                id: 1,
                name: String::new(),
                kind: SourceKind::Window,
                selection: estela_capture::session::SourceSelection::SystemPicker,
            },
        ]);
    }
    super::capture::x11::sources()
}

pub fn prepare(source: &Source, fps: u32) -> Result<Box<dyn Session>> {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        super::capture::portal::prepare(source, fps)
    } else {
        super::capture::x11::prepare(source)
    }
}

pub use super::input::shortcuts::listen as listen_shortcuts;

pub fn exclude_window(_id: u64) -> bool {
    false
}
