#![cfg(target_os = "linux")]

use std::time::Duration;

use x11rb::{
    connection::Connection,
    protocol::xproto::{ConnectionExt, CreateWindowAux, WindowClass},
    COPY_FROM_PARENT,
};

use estela_capture::session::{Source, SourceKind};
use estela_platform_linux::capture::x11;

#[test]
#[ignore = "Requires an isolated X11 server with ESTELA_TEST_X11=1 and DISPLAY set."]
fn captures_a_window_without_an_overlapping_window() {
    assert_eq!(std::env::var("ESTELA_TEST_X11").as_deref(), Ok("1"));
    let (connection, screen) = x11rb::connect(None).unwrap();
    let screen = &connection.setup().roots[screen];
    let window = connection.generate_id().unwrap();
    connection
        .create_window(
            COPY_FROM_PARENT as u8,
            window,
            screen.root,
            20,
            20,
            160,
            120,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().background_pixel(screen.white_pixel),
        )
        .unwrap()
        .check()
        .unwrap();
    connection.map_window(window).unwrap().check().unwrap();
    connection.flush().unwrap();
    let source = Source {
        id: u64::from(window),
        name: "Capture fixture".into(),
        kind: SourceKind::Window,
        selection: estela_capture::session::SourceSelection::Direct,
    };
    let mut session = x11::prepare(&source).unwrap();
    let overlay = connection.generate_id().unwrap();
    connection
        .create_window(
            COPY_FROM_PARENT as u8,
            overlay,
            screen.root,
            20,
            20,
            160,
            120,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().background_pixel(screen.black_pixel),
        )
        .unwrap()
        .check()
        .unwrap();
    connection.map_window(overlay).unwrap().check().unwrap();
    connection.flush().unwrap();
    let frame = session
        .next_frame(Duration::from_millis(10))
        .unwrap()
        .unwrap();
    assert_eq!((frame.width, frame.height), (160, 120));
    assert_eq!(&frame.rgba[..4], &[255, 255, 255, 255]);
    drop(session);
    connection.destroy_window(overlay).unwrap();
    connection.destroy_window(window).unwrap();
    connection.flush().unwrap();
}
