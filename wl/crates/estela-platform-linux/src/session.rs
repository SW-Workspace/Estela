#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Session {
    Wayland,
    X11,
    Unknown,
}
pub fn detect() -> Session {
    match std::env::var("XDG_SESSION_TYPE").ok().as_deref() {
        Some("wayland") => Session::Wayland,
        Some("x11") => Session::X11,
        _ if std::env::var_os("WAYLAND_DISPLAY").is_some() => Session::Wayland,
        _ if std::env::var_os("DISPLAY").is_some() => Session::X11,
        _ => Session::Unknown,
    }
}
