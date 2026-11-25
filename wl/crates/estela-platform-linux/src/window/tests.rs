use x11rb::{
    connection::Connection,
    protocol::xproto::{ConnectionExt, CreateWindowAux, WindowClass},
    COPY_FROM_PARENT, NONE,
};

use estela_capture::window::InputArea;

use super::*;

#[test]
#[ignore = "Requires an isolated X11 server with ESTELA_TEST_X11=1 and DISPLAY set."]
fn transparent_regions_pass_pointer_input_to_the_window_below() {
    assert_eq!(std::env::var("ESTELA_TEST_X11").as_deref(), Ok("1"));
    let (connection, screen) = x11rb::connect(None).unwrap();
    let root = connection.setup().roots[screen].root;
    let below = connection.generate_id().unwrap();
    let recorder = connection.generate_id().unwrap();
    for window in [below, recorder] {
        connection
            .create_window(
                COPY_FROM_PARENT as u8,
                window,
                root,
                400,
                300,
                600,
                440,
                0,
                WindowClass::INPUT_OUTPUT,
                0,
                &CreateWindowAux::new(),
            )
            .unwrap()
            .check()
            .unwrap();
        connection.map_window(window).unwrap().check().unwrap();
    }
    connection.flush().unwrap();
    let target = |x: i16, y: i16| {
        connection
            .warp_pointer(NONE, root, 0, 0, 0, 0, x + 400, y + 300)
            .unwrap()
            .check()
            .unwrap();
        connection
            .query_pointer(root)
            .unwrap()
            .reply()
            .unwrap()
            .child
    };
    let bar = InputArea {
        x: 16,
        y: 16,
        width: 568,
        height: 56,
        radius: 28,
    };
    x11(recorder, &WindowInput::Areas(vec![bar]), 1.0).unwrap();
    assert_eq!(target(200, 44), recorder);
    assert_eq!(target(17, 17), below);
    assert_eq!(target(200, 200), below);
    let panel = InputArea {
        x: 120,
        y: 84,
        width: 360,
        height: 340,
        radius: 14,
    };
    x11(recorder, &WindowInput::Areas(vec![bar, panel]), 1.0).unwrap();
    assert_eq!(target(200, 200), recorder);
    assert_eq!(target(200, 78), below);
    x11(recorder, &WindowInput::Areas(vec![bar]), 1.5).unwrap();
    assert_eq!(target(300, 100), recorder);
    assert_eq!(target(300, 120), below);
    x11(recorder, &WindowInput::Areas(Vec::new()), 1.0).unwrap();
    assert_eq!(target(200, 44), below);
    x11(recorder, &WindowInput::Entire, 1.0).unwrap();
    assert_eq!(target(200, 200), recorder);
    for window in [recorder, below] {
        connection.destroy_window(window).unwrap().check().unwrap();
    }
}
