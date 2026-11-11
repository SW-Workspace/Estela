use std::{
    fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use estela_engine::preview;
use estela_format::{
    reader,
    tracks::{Recording, ZoomTrigger},
    writer,
};
use estela_media_ffmpeg::{decode::Decoder, encode::Encoder};
use estela_model::{Button, Event, EventKind, Time};

#[test]
#[ignore = "Requires dynamically linked LGPL FFmpeg on PATH."]
fn records_reopens_seeks_and_previews_automatic_zoom_without_changing_original() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "estela-roundtrip-{}-{unique}.estela",
        std::process::id()
    ));
    let size = (64, 48);
    let mut encoder = Encoder::start(&path, size, 30).unwrap();
    for frame in 0..60 {
        let mut rgba = Vec::new();
        for y in 0..size.1 {
            for x in 0..size.0 {
                rgba.extend_from_slice(&[(x * 4) as u8, (y * 5) as u8, frame * 3, 255]);
            }
        }
        encoder.write(&rgba).unwrap();
    }
    encoder.finish().unwrap();
    let events = vec![
        Event {
            t: Time(0),
            kind: EventKind::Move { x: 40, y: 30 },
        },
        Event {
            t: Time(500_000_000),
            kind: EventKind::Down {
                button: Button::Left,
            },
        },
    ];
    let mut recording = Recording {
        version: 1,
        title: "Round trip".into(),
        width: size.0,
        height: size.1,
        fps: 30,
        frames: 60,
        duration: Time(2_000_000_000),
        cursor_embedded: false,
        zoom_trigger: ZoomTrigger::Clicks,
        events,
        zooms: Vec::new(),
    };
    recording.zooms =
        estela_motion::autozoom::generate(&recording.events, size, recording.duration);
    writer::finalize(&path, &recording).unwrap();
    let original_file = fs::read(&path).unwrap();
    let reopened = reader::open(&path).unwrap();
    assert_eq!(reopened.zooms.len(), 1);
    let mut decoder = Decoder::start(&path, size, 0.0).unwrap();
    let mut count = 0;
    while decoder.next_frame().unwrap().is_some() {
        count += 1;
    }
    assert_eq!(count, 60);
    let mut decoder = Decoder::start(&path, size, 0.75).unwrap();
    let original = decoder.next_frame().unwrap().unwrap();
    drop(decoder);
    let zoomed = preview::compose(&original, size, &reopened, Time(750_000_000), true);
    let unzoomed = preview::compose(&original, size, &reopened, Time(750_000_000), false);
    assert_ne!(zoomed, unzoomed);
    let player = preview::Player::open(path.clone(), reopened.clone());
    let first = player.events.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(
        matches!(first, preview::Event::Frame { seconds: 0.0, .. }),
        "{first:?}"
    );
    player.send(preview::Command::Seek(0.75));
    let seeked = player.events.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(
        matches!(seeked, preview::Event::Frame { seconds: 0.75, .. }),
        "{seeked:?}"
    );
    player.send(preview::Command::Stop);
    assert!(matches!(
        player.events.recv_timeout(Duration::from_secs(5)),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected)
    ));
    drop(player);
    assert_eq!(fs::read(&path).unwrap(), original_file);
    fs::remove_file(path).unwrap();
}
