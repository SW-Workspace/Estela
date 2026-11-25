use std::{
    fs::{self, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use estela_format::{
    consts::SLOT_SIZE,
    reader,
    tracks::{Recording, ZoomTrigger},
    writer,
};
use estela_model::Time;

fn fixture() -> (PathBuf, Recording) {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "estela-format-{}-{unique}.estela",
        std::process::id()
    ));
    fs::write(
        &path,
        [
            0, 0, 0, 24, b'f', b't', b'y', b'p', b'i', b's', b'o', b'm', 0, 0, 0, 0, b'i', b's',
            b'o', b'm', b'm', b'p', b'4', b'1',
        ],
    )
    .unwrap();
    let recording = Recording {
        version: 1,
        title: "Test".into(),
        width: 64,
        height: 48,
        fps: 30,
        frames: 60,
        duration: Time(2_000_000_000),
        cursor_embedded: false,
        zoom_trigger: ZoomTrigger::Clicks,
        events: Vec::new(),
        zooms: Vec::new(),
    };
    (path, recording)
}

#[test]
fn reopens_recording_and_falls_back_from_a_damaged_slot() {
    let (path, recording) = fixture();
    writer::finalize(&path, &recording).unwrap();
    assert_eq!(reader::open(&path).unwrap().frames, 60);
    let mut file = OpenOptions::new().write(true).open(&path).unwrap();
    file.seek(SeekFrom::End(-(SLOT_SIZE as i64) + 24)).unwrap();
    file.write_all(&u64::MAX.to_be_bytes()).unwrap();
    assert_eq!(reader::open(&path).unwrap().title, "Test");
    fs::remove_file(path).unwrap();
}

#[test]
fn rejects_tampered_metadata_and_foreign_files() {
    let (path, recording) = fixture();
    assert!(reader::open(&path).is_err());
    writer::finalize(&path, &recording).unwrap();
    let mut file = OpenOptions::new().write(true).open(&path).unwrap();
    file.seek(SeekFrom::Start(48)).unwrap();
    file.write_all(b"!").unwrap();
    assert!(reader::open(&path).is_err());
    fs::remove_file(path).unwrap();
}

#[test]
fn rejects_invalid_dimensions_before_writing() {
    let (path, mut recording) = fixture();
    recording.width = 0;
    let original = fs::read(&path).unwrap();
    assert!(writer::finalize(&path, &recording).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::remove_file(path).unwrap();
}
