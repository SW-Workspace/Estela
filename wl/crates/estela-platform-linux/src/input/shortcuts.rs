use std::sync::mpsc::Sender;

use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
use futures_util::StreamExt;
use x11rb::{
    connection::Connection,
    protocol::xproto::{ConnectionExt, GrabMode, ModMask},
    protocol::Event,
};

use estela_capture::session::Shortcut;

pub fn listen(sender: Sender<Shortcut>, descriptions: [String; 2]) {
    std::thread::spawn(move || {
        let result = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            crate::portal_runtime::runtime()
                .and_then(|runtime| runtime.block_on(portal(&sender, descriptions)))
        } else {
            x11(&sender)
        };
        if result.is_err() {
            let _ = sender.send(Shortcut::Unavailable);
        }
    });
}

async fn portal(
    sender: &Sender<Shortcut>,
    descriptions: [String; 2],
) -> std::result::Result<(), String> {
    crate::portal_runtime::register_application().await;
    let proxy = GlobalShortcuts::new()
        .await
        .map_err(|error| error.to_string())?;
    let session = proxy
        .create_session(Default::default())
        .await
        .map_err(|error| error.to_string())?;
    let shortcuts = [
        NewShortcut::new("record", &descriptions[0]).preferred_trigger("CTRL+SHIFT+8"),
        NewShortcut::new("pause", &descriptions[1]).preferred_trigger("CTRL+SHIFT+9"),
    ];
    proxy
        .bind_shortcuts(&session, &shortcuts, None, Default::default())
        .await
        .map_err(|error| error.to_string())?
        .response()
        .map_err(|error| error.to_string())?;
    let session_path = serde_json::to_value(&session).map_err(|error| error.to_string())?;
    let mut events = proxy
        .receive_activated()
        .await
        .map_err(|error| error.to_string())?;
    sender
        .send(Shortcut::Ready)
        .map_err(|error| error.to_string())?;
    while let Some(event) = events.next().await {
        if Some(event.session_handle().as_str()) != session_path.as_str() {
            continue;
        }
        let shortcut = if event.shortcut_id() == "record" {
            Shortcut::Record
        } else {
            Shortcut::Pause
        };
        if sender.send(shortcut).is_err() {
            break;
        }
    }
    let _ = session.close().await;
    Ok(())
}

fn x11(sender: &Sender<Shortcut>) -> std::result::Result<(), String> {
    let (connection, screen) = x11rb::connect(None).map_err(|error| error.to_string())?;
    let setup = connection.setup();
    let root = setup.roots[screen].root;
    let keys = connection
        .get_keyboard_mapping(setup.min_keycode, setup.max_keycode - setup.min_keycode + 1)
        .map_err(|error| error.to_string())?
        .reply()
        .map_err(|error| error.to_string())?;
    let find = |symbol| {
        keys.keysyms
            .chunks(keys.keysyms_per_keycode as usize)
            .position(|symbols| symbols.contains(&symbol))
            .map(|index| setup.min_keycode + index as u8)
            .ok_or_else(|| "Shortcut key unavailable".to_string())
    };
    let record = find(0x38)?;
    let pause = find(0x39)?;
    for code in [record, pause] {
        for extra in [
            0,
            u16::from(ModMask::LOCK),
            u16::from(ModMask::M2),
            u16::from(ModMask::LOCK) | u16::from(ModMask::M2),
        ] {
            let modifiers = ModMask::from(u16::from(ModMask::CONTROL | ModMask::SHIFT) | extra);
            connection
                .grab_key(
                    false,
                    root,
                    modifiers,
                    code,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                )
                .map_err(|error| error.to_string())?
                .check()
                .map_err(|error| error.to_string())?;
        }
    }
    connection.flush().map_err(|error| error.to_string())?;
    sender
        .send(Shortcut::Ready)
        .map_err(|error| error.to_string())?;
    loop {
        if let Event::KeyPress(event) = connection
            .wait_for_event()
            .map_err(|error| error.to_string())?
        {
            let action = if event.detail == record {
                Shortcut::Record
            } else if event.detail == pause {
                Shortcut::Pause
            } else {
                continue;
            };
            if sender.send(action).is_err() {
                return Ok(());
            }
        }
    }
}
