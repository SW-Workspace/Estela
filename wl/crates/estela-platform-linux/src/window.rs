use raw_window_handle::{DisplayHandle, RawDisplayHandle, RawWindowHandle, WindowHandle};
use x11rb::{
    connection::Connection,
    protocol::{
        shape::{ConnectionExt as _, SK, SO},
        xproto::{ClipOrdering, Rectangle},
    },
};

use estela_capture::{window::WindowInput, Error, Result};

mod wayland;

pub fn configure(
    window: WindowHandle<'_>,
    display: DisplayHandle<'_>,
    input: &WindowInput,
    scale: f32,
) -> Result<bool> {
    match (window.as_raw(), display.as_raw()) {
        (RawWindowHandle::Wayland(window), RawDisplayHandle::Wayland(display)) => {
            wayland::configure(window.surface.as_ptr(), display.display.as_ptr(), input)
        }
        (RawWindowHandle::Xlib(window), _) => x11(window.window as u32, input, scale),
        (RawWindowHandle::Xcb(window), _) => x11(window.window.get(), input, scale),
        _ => Err(Error::Backend("Unsupported window handle".into())),
    }?;
    Ok(false)
}

pub fn release(display: DisplayHandle<'_>) {
    if let RawDisplayHandle::Wayland(display) = display.as_raw() {
        wayland::release(display.display.as_ptr());
    }
}

fn x11(window: u32, input: &WindowInput, scale: f32) -> Result<()> {
    let (connection, _) = x11rb::connect(None).map_err(backend)?;
    match input {
        WindowInput::Entire => {
            for kind in [SK::INPUT, SK::BOUNDING] {
                connection
                    .shape_mask(SO::SET, kind, window, 0, 0, x11rb::NONE)
                    .map_err(backend)?
                    .check()
                    .map_err(backend)?;
            }
        }
        WindowInput::Areas(areas) => {
            let rectangles: Vec<_> = areas
                .iter()
                .flat_map(|area| area.rectangles(scale))
                .filter(|area| area.width > 0 && area.height > 0)
                .map(|area| Rectangle {
                    x: area.x as i16,
                    y: area.y as i16,
                    width: area.width as u16,
                    height: area.height as u16,
                })
                .collect();
            for kind in [SK::INPUT, SK::BOUNDING] {
                connection
                    .shape_rectangles(
                        SO::SET,
                        kind,
                        ClipOrdering::UNSORTED,
                        window,
                        0,
                        0,
                        &rectangles,
                    )
                    .map_err(backend)?
                    .check()
                    .map_err(backend)?;
            }
        }
    }
    connection.flush().map_err(backend)
}

fn backend(error: impl std::fmt::Display) -> Error {
    Error::Backend(error.to_string())
}

#[cfg(test)]
mod tests;

pub fn presentation_backend() -> &'static str {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        "wgpu"
    } else {
        "tiny-skia"
    }
}
