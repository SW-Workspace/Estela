use raw_window_handle::{DisplayHandle, RawWindowHandle, WindowHandle};
use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{
        CombineRgn, CreateRectRgn, CreateRoundRectRgn, DeleteObject, SetWindowRgn, RGN_OR,
    },
};

use estela_capture::{window::WindowInput, Error, Result};

pub fn configure(
    window: WindowHandle<'_>,
    _: DisplayHandle<'_>,
    input: &WindowInput,
    scale: f32,
) -> Result<bool> {
    let RawWindowHandle::Win32(window) = window.as_raw() else {
        return Err(Error::Backend("Unsupported window handle".into()));
    };
    let hwnd = HWND(window.hwnd.get() as *mut _);
    // SAFETY: The borrowed window is live and GDI regions are owned in this call.
    unsafe {
        match input {
            WindowInput::Entire => {
                if SetWindowRgn(hwnd, None, true) == 0 {
                    return Err(std::io::Error::last_os_error().into());
                }
            }
            WindowInput::Areas(areas) => {
                let combined = CreateRectRgn(0, 0, 0, 0);
                if combined.0.is_null() {
                    return Err(std::io::Error::last_os_error().into());
                }
                for area in areas {
                    let x = (area.x as f32 * scale).round() as i32;
                    let y = (area.y as f32 * scale).round() as i32;
                    let width = (area.width as f32 * scale).round() as i32;
                    let height = (area.height as f32 * scale).round() as i32;
                    let diameter = (area.radius as f32 * scale * 2.0).round() as i32;
                    let region =
                        CreateRoundRectRgn(x, y, x + width, y + height, diameter, diameter);
                    if region.0.is_null() {
                        let _ = DeleteObject(combined.into());
                        return Err(std::io::Error::last_os_error().into());
                    }
                    let result = CombineRgn(Some(combined), Some(combined), Some(region), RGN_OR);
                    let _ = DeleteObject(region.into());
                    if result.0 == 0 {
                        let _ = DeleteObject(combined.into());
                        return Err(std::io::Error::last_os_error().into());
                    }
                }
                if SetWindowRgn(hwnd, Some(combined), true) == 0 {
                    let _ = DeleteObject(combined.into());
                    return Err(std::io::Error::last_os_error().into());
                }
            }
        }
    }
    Ok(super::adapter::exclude_window(window.hwnd.get() as u64))
}

pub fn release(_: DisplayHandle<'_>) {}

pub fn presentation_backend() -> &'static str {
    "wgpu"
}
