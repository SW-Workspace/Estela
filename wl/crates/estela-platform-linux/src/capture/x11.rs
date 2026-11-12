use std::time::Duration;

use x11rb::{
    connection::Connection,
    protocol::{
        composite::{ConnectionExt as _, Redirect},
        randr::ConnectionExt as _,
        xproto::{AtomEnum, ConnectionExt as _, ImageFormat, Window},
    },
    rust_connection::RustConnection,
};

use estela_capture::{
    session::{Frame, Pointer, Session, Source, SourceKind},
    Error, Result,
};

fn backend(error: impl std::fmt::Display) -> Error {
    Error::Backend(error.to_string())
}

pub fn sources() -> Result<Vec<Source>> {
    let (connection, screen) = x11rb::connect(None).map_err(backend)?;
    let root = connection.setup().roots[screen].root;
    let mut sources = Vec::new();
    if let Ok(monitors) = connection
        .randr_get_monitors(root, true)
        .map_err(backend)?
        .reply()
    {
        for monitor in monitors.monitors {
            let name = connection
                .get_atom_name(monitor.name)
                .map_err(backend)?
                .reply()
                .map_err(backend)?
                .name;
            sources.push(Source {
                id: u64::from(monitor.name),
                name: String::from_utf8_lossy(&name).into_owned(),
                kind: SourceKind::Display,
                selection: estela_capture::session::SourceSelection::Direct,
            });
        }
    }
    if sources.is_empty() {
        sources.push(Source {
            id: u64::from(root),
            name: "X11".into(),
            kind: SourceKind::Display,
            selection: estela_capture::session::SourceSelection::Direct,
        });
    }
    let clients = atom(&connection, b"_NET_CLIENT_LIST")?;
    let reply = connection
        .get_property(false, root, clients, AtomEnum::WINDOW, 0, 4096)
        .map_err(backend)?
        .reply()
        .map_err(backend)?;
    for window in reply.value32().into_iter().flatten() {
        let pid = property(&connection, window, b"_NET_WM_PID", AtomEnum::CARDINAL)?;
        if pid.value32().and_then(|mut values| values.next()) == Some(std::process::id()) {
            continue;
        }
        let title = property(&connection, window, b"_NET_WM_NAME", AtomEnum::ANY)?;
        let name = String::from_utf8_lossy(&title.value).into_owned();
        if !name.is_empty() {
            sources.push(Source {
                id: u64::from(window),
                name,
                kind: SourceKind::Window,
                selection: estela_capture::session::SourceSelection::Direct,
            });
        }
    }
    Ok(sources)
}

fn atom(connection: &RustConnection, name: &[u8]) -> Result<u32> {
    Ok(connection
        .intern_atom(false, name)
        .map_err(backend)?
        .reply()
        .map_err(backend)?
        .atom)
}

fn property(
    connection: &RustConnection,
    window: Window,
    name: &[u8],
    kind: AtomEnum,
) -> Result<x11rb::protocol::xproto::GetPropertyReply> {
    connection
        .get_property(false, window, atom(connection, name)?, kind, 0, 4096)
        .map_err(backend)?
        .reply()
        .map_err(backend)
}

struct Capture {
    connection: RustConnection,
    drawable: Window,
    pointer_window: Window,
    redirected: bool,
    bounds: (i16, i16, u16, u16),
}

pub fn prepare(source: &Source) -> Result<Box<dyn Session>> {
    let (connection, screen) = x11rb::connect(None).map_err(backend)?;
    let root = connection.setup().roots[screen].root;
    let mut capture = Capture {
        connection,
        drawable: root,
        pointer_window: root,
        redirected: false,
        bounds: (0, 0, 0, 0),
    };
    if source.kind == SourceKind::Window {
        capture.drawable = source.id as Window;
        capture.pointer_window = capture.drawable;
        let geometry = capture
            .connection
            .get_geometry(capture.drawable)
            .map_err(backend)?
            .reply()
            .map_err(backend)?;
        capture.bounds = (0, 0, geometry.width, geometry.height);
        let pixmap = capture.connection.generate_id().map_err(backend)?;
        if capture
            .connection
            .composite_name_window_pixmap(capture.pointer_window, pixmap)
            .map_err(backend)?
            .check()
            .is_err()
        {
            capture
                .connection
                .composite_redirect_window(capture.pointer_window, Redirect::AUTOMATIC)
                .map_err(backend)?
                .check()
                .map_err(backend)?;
            capture.redirected = true;
            capture
                .connection
                .composite_name_window_pixmap(capture.pointer_window, pixmap)
                .map_err(backend)?
                .check()
                .map_err(backend)?;
        }
        capture.drawable = pixmap;
    } else {
        let geometry = capture
            .connection
            .get_geometry(root)
            .map_err(backend)?
            .reply()
            .map_err(backend)?;
        capture.bounds = (0, 0, geometry.width, geometry.height);
        if source.id != u64::from(root) {
            let monitors = capture
                .connection
                .randr_get_monitors(root, true)
                .map_err(backend)?
                .reply()
                .map_err(backend)?;
            let monitor = monitors
                .monitors
                .iter()
                .find(|monitor| u64::from(monitor.name) == source.id)
                .ok_or_else(|| backend("Display disconnected"))?;
            capture.bounds = (monitor.x, monitor.y, monitor.width, monitor.height);
        }
    }
    Ok(Box::new(capture))
}

impl Session for Capture {
    fn next_frame(&mut self, _timeout: Duration) -> Result<Option<Frame>> {
        let (x, y, width, height) = self.bounds;
        if self.pointer_window != self.drawable {
            let current = self
                .connection
                .get_geometry(self.pointer_window)
                .map_err(backend)?
                .reply()
                .map_err(backend)?;
            if current.width != width || current.height != height {
                return Err(backend("Source dimensions changed"));
            }
        }
        let image = self
            .connection
            .get_image(
                ImageFormat::Z_PIXMAP,
                self.drawable,
                x,
                y,
                width,
                height,
                u32::MAX,
            )
            .map_err(backend)?
            .reply()
            .map_err(backend)?;
        if image.data.len() != usize::from(width) * usize::from(height) * 4 {
            return Err(backend("Unsupported X11 pixel format"));
        }
        let mut rgba = image.data;
        for pixel in rgba.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
            pixel[3] = 255;
        }
        Ok(Some(Frame {
            width: u32::from(width),
            height: u32::from(height),
            rgba,
        }))
    }

    fn pointer(&self) -> Result<Pointer> {
        let pointer = self
            .connection
            .query_pointer(self.pointer_window)
            .map_err(backend)?
            .reply()
            .map_err(backend)?;
        let mask = u16::from(pointer.mask);
        Ok(Pointer {
            x: i32::from(pointer.win_x - self.bounds.0),
            y: i32::from(pointer.win_y - self.bounds.1),
            buttons: [mask & 256 != 0, mask & 1024 != 0, mask & 512 != 0],
            available: pointer.same_screen,
        })
    }

    fn cursor_embedded(&self) -> bool {
        false
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        if self.drawable != self.pointer_window {
            let _ = self.connection.free_pixmap(self.drawable);
        }
        if self.redirected {
            let _ = self
                .connection
                .composite_unredirect_window(self.pointer_window, Redirect::AUTOMATIC);
        }
        let _ = self.connection.flush();
    }
}
