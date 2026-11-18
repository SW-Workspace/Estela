use std::{sync::mpsc::Sender, time::Duration};

use windows::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{
            RegisterHotKey, UnregisterHotKey, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
        },
        WindowsAndMessaging::{PeekMessageW, MSG, PM_REMOVE, WM_HOTKEY},
    },
};

use estela_capture::session::Shortcut;

pub fn listen(sender: Sender<Shortcut>, _descriptions: [String; 2]) {
    std::thread::spawn(move || {
        // SAFETY: Thread hotkeys have no window pointer and use unique IDs.
        let registered = unsafe {
            RegisterHotKey(None, 1, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, 0x38).and_then(|()| {
                RegisterHotKey(None, 2, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, 0x39)
            })
        };
        if registered.is_err() {
            let _ = sender.send(Shortcut::Unavailable);
            return;
        }
        let _ = sender.send(Shortcut::Ready);
        loop {
            let mut message = MSG::default();
            // SAFETY: The message buffer is writable and only this thread is polled.
            if unsafe {
                PeekMessageW(
                    &mut message,
                    Some(HWND::default()),
                    WM_HOTKEY,
                    WM_HOTKEY,
                    PM_REMOVE,
                )
            }
            .as_bool()
            {
                let shortcut = if message.wParam.0 == 1 {
                    Shortcut::Record
                } else {
                    Shortcut::Pause
                };
                if sender.send(shortcut).is_err() {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        // SAFETY: Both registrations belong to this thread.
        unsafe {
            let _ = UnregisterHotKey(None, 1);
            let _ = UnregisterHotKey(None, 2);
        }
    });
}
