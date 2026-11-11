use std::{
    sync::mpsc::{self, Receiver, SyncSender},
    time::Duration,
};

use windows::Win32::{
    Foundation::POINT,
    Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFO},
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON},
        WindowsAndMessaging::{GetCursorPos, GetWindowThreadProcessId, WindowFromPoint},
    },
};
use windows_capture::{
    capture::{CaptureControl, Context, GraphicsCaptureApiHandler},
    frame::Frame as NativeFrame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
    window::Window,
};

use estela_capture::{
    session::{Frame, Pointer, Session, Source, SourceKind},
    Error, Result,
};

type CaptureError = Box<dyn std::error::Error + Send + Sync>;

fn backend(error: impl std::fmt::Display) -> Error {
    Error::Backend(error.to_string())
}

pub fn sources() -> Result<Vec<Source>> {
    let mut sources = Vec::new();
    for monitor in Monitor::enumerate().map_err(backend)? {
        sources.push(Source {
            id: monitor.as_raw_hmonitor() as u64,
            name: monitor.name().map_err(backend)?,
            kind: SourceKind::Display,
            selection: estela_capture::session::SourceSelection::Direct,
        });
    }
    for window in Window::enumerate().map_err(backend)? {
        if window.process_id().map_err(backend)? == std::process::id() {
            continue;
        }
        sources.push(Source {
            id: window.as_raw_hwnd() as u64,
            name: window.title().map_err(backend)?,
            kind: SourceKind::Window,
            selection: estela_capture::session::SourceSelection::Direct,
        });
    }
    Ok(sources)
}

pub struct Handler {
    sender: SyncSender<Frame>,
}

impl GraphicsCaptureApiHandler for Handler {
    type Flags = SyncSender<Frame>;
    type Error = CaptureError;
    fn new(context: Context<Self::Flags>) -> std::result::Result<Self, CaptureError> {
        Ok(Self {
            sender: context.flags,
        })
    }
    fn on_frame_arrived(
        &mut self,
        frame: &mut NativeFrame,
        control: InternalCaptureControl,
    ) -> std::result::Result<(), CaptureError> {
        let (width, height) = (frame.width(), frame.height());
        let buffer = frame.buffer()?;
        let mut storage = Vec::new();
        let rgba = buffer.as_nopadding_buffer(&mut storage).to_vec();
        if let Err(mpsc::TrySendError::Disconnected(_)) = self.sender.try_send(Frame {
            width,
            height,
            rgba,
        }) {
            control.stop();
        }
        Ok(())
    }
}

struct Capture {
    frames: Receiver<Frame>,
    control: Option<CaptureControl<Handler, CaptureError>>,
    source: Source,
}

pub fn prepare(source: &Source, _fps: u32) -> Result<Box<dyn Session>> {
    let (sender, frames) = mpsc::sync_channel(2);
    let control = match source.kind {
        SourceKind::Display => {
            start_capture(Monitor::from_raw_hmonitor(source.id as *mut _), sender)?
        }
        SourceKind::Window => start_capture(Window::from_raw_hwnd(source.id as *mut _), sender)?,
    };
    Ok(Box::new(Capture {
        frames,
        control: Some(control),
        source: source.clone(),
    }))
}

fn start_capture<T>(
    item: T,
    sender: SyncSender<Frame>,
) -> Result<CaptureControl<Handler, CaptureError>>
where
    T: TryInto<windows_capture::settings::GraphicsCaptureItemType> + Send + 'static,
{
    let settings = Settings::new(
        item,
        CursorCaptureSettings::WithoutCursor,
        DrawBorderSettings::Default,
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Rgba8,
        sender,
    );
    Handler::start_free_threaded(settings).map_err(backend)
}

impl Session for Capture {
    fn next_frame(&mut self, timeout: Duration) -> Result<Option<Frame>> {
        match self.frames.recv_timeout(timeout) {
            Ok(frame) => Ok(Some(frame)),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if self
                    .control
                    .as_ref()
                    .is_some_and(|control| control.is_finished())
                {
                    return Err(backend("Capture source closed"));
                }
                Ok(None)
            }
            Err(error) => Err(backend(error)),
        }
    }
    fn pointer(&self) -> Result<Pointer> {
        let mut position = POINT::default();
        // SAFETY: The API receives a valid writable POINT for this call.
        unsafe { GetCursorPos(&mut position) }.map_err(backend)?;
        let mut process_id = 0;
        // SAFETY: The hit-test handle and writable process ID are used immediately.
        unsafe {
            GetWindowThreadProcessId(WindowFromPoint(position), Some(&mut process_id));
        }
        if process_id == std::process::id() {
            return Ok(Pointer::default());
        }
        let (x, y) = if self.source.kind == SourceKind::Window {
            let rect = Window::from_raw_hwnd(self.source.id as *mut _)
                .rect()
                .map_err(backend)?;
            (rect.left, rect.top)
        } else {
            let mut info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            // SAFETY: The selected monitor handle and writable structure are valid.
            if !unsafe { GetMonitorInfoW(HMONITOR(self.source.id as *mut _), &mut info) }.as_bool()
            {
                return Err(backend("Monitor disconnected"));
            }
            (info.rcMonitor.left, info.rcMonitor.top)
        };
        // SAFETY: These calls query button state without retaining any pointers.
        let buttons = unsafe {
            [
                GetAsyncKeyState(VK_LBUTTON.0 as i32) < 0,
                GetAsyncKeyState(VK_RBUTTON.0 as i32) < 0,
                GetAsyncKeyState(VK_MBUTTON.0 as i32) < 0,
            ]
        };
        Ok(Pointer {
            x: position.x - x,
            y: position.y - y,
            buttons,
            available: true,
        })
    }
    fn cursor_embedded(&self) -> bool {
        false
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        if let Some(control) = self.control.take() {
            let _ = control.stop();
        }
    }
}

pub use super::input::shortcuts::listen as listen_shortcuts;

pub fn exclude_window(id: u64) -> bool {
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE},
    };
    // SAFETY: Iced supplies the native handle of its own live window.
    unsafe { SetWindowDisplayAffinity(HWND(id as *mut _), WDA_EXCLUDEFROMCAPTURE) }.is_ok()
}
