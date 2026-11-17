use std::time::{SystemTime, UNIX_EPOCH};

use estela_media_ffmpeg::{decode::Decoder, encode::Encoder};

#[test]
#[ignore = "Requires dynamically linked LGPL FFmpeg."]
fn encodes_and_decodes_recorded_frames() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "estela-media-runtime-{}-{unique}.mp4",
        std::process::id(),
    ));
    estela_media_ffmpeg::encode::available().unwrap();
    let size = (16, 16);
    let mut encoder = Encoder::start(&path, size, 30).unwrap();
    for _ in 0..30 {
        encoder.write(&[128; 16 * 16 * 4]).unwrap();
    }
    encoder.finish().unwrap();
    let mut decoder = Decoder::start(&path, size, 0.0).unwrap();
    let mut frames = 0;
    while let Some(frame) = decoder.next_frame().unwrap() {
        assert_eq!(frame.len(), 16 * 16 * 4);
        frames += 1;
    }
    assert_eq!(frames, 30);
    drop(decoder);
    let mut decoder = Decoder::start(&path, size, 0.5).unwrap();
    assert!(decoder.next_frame().unwrap().is_some());
    drop(decoder);
    std::fs::remove_file(path).unwrap();
}
