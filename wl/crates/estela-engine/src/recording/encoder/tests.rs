use std::time::{SystemTime, UNIX_EPOCH};

use estela_media::recording::RecordingFrame;
use estela_media_ffmpeg::decode::Decoder;

use super::*;

#[test]
#[ignore = "Requires FFmpeg to encode and decode real recording frames."]
fn unavailable_hardware_falls_back_without_overwriting_destination() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "estela-fallback-{}-{unique}.mp4",
        std::process::id()
    ));
    let config = RecordingConfig {
        width: 65,
        height: 49,
        fps: 30,
    };
    let candidates = vec![
        RecordingBackend::Vaapi {
            device: path.with_extension("missing-device"),
        },
        RecordingBackend::Software,
    ];
    let mut encoder = prepare_with_backends(config, candidates)
        .unwrap()
        .start(&path)
        .unwrap();
    assert_eq!(encoder.backend(), &RecordingBackend::Software);
    let rgba = vec![128; config.frame_bytes().unwrap()];
    assert!(encoder
        .write(RecordingFrame {
            rgba: &rgba,
            pts: config.timestamp(1)
        })
        .is_err());
    assert!(encoder
        .write(RecordingFrame {
            rgba: &rgba[..4],
            pts: config.timestamp(0)
        })
        .is_err());
    for frame in 0..30 {
        encoder
            .write(RecordingFrame {
                rgba: &rgba,
                pts: config.timestamp(frame),
            })
            .unwrap();
    }
    encoder.finish().unwrap();
    let original = std::fs::read(&path).unwrap();
    assert!(
        prepare_with_backends(config, vec![RecordingBackend::Software])
            .unwrap()
            .start(&path)
            .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let mut decoder = Decoder::start(&path, (config.width, config.height), 0.0).unwrap();
    let mut frames = 0;
    while let Some(frame) = decoder.next_frame().unwrap() {
        assert_eq!(frame.len(), rgba.len());
        frames += 1;
    }
    assert_eq!(frames, 30);
    drop(decoder);
    std::fs::remove_file(path).unwrap();
}
