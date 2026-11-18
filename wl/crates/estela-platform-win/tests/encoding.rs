#![cfg(windows)]

use std::{
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use estela_media::recording::{RecordingBackend, RecordingConfig, RecordingFrame};
use estela_platform_win::encoding::{native_factory, start_native, validate_fragmented_media};

#[test]
fn invalid_native_configuration_does_not_create_a_file() {
    let path = temporary_path("invalid");
    let config = RecordingConfig {
        width: 0,
        height: 2,
        fps: 30,
    };
    assert!(start_native(&path, config).is_err());
    assert!(!path.exists());
    assert!(native_factory(&RecordingBackend::Software).is_none());
}

#[test]
#[ignore = "Requires native Windows Media Foundation and LGPL FFmpeg decoding."]
fn native_h264_roundtrip_preserves_frames_color_and_existing_destinations() {
    for fps in [30, 60] {
        let path = temporary_path("native");
        let config = RecordingConfig {
            width: 321,
            height: 241,
            fps,
        };
        let mut encoder = start_native(&path, config).unwrap();
        assert_eq!(encoder.backend(), &RecordingBackend::MediaFoundation);
        let rgba = [255, 0, 0, 255].repeat(config.frame_bytes().unwrap() / 4);
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
        for frame in 0..u64::from(fps) {
            encoder
                .write(RecordingFrame {
                    rgba: &rgba,
                    pts: config.timestamp(frame),
                })
                .unwrap();
        }
        encoder.finish().unwrap();
        validate_fragmented_media(&path).unwrap();
        let encoded = std::fs::read(&path).unwrap();
        assert!(encoded.windows(4).any(|bytes| bytes == b"avc1"));
        assert!(start_native(&path, config).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), encoded);
        verify_decoded(&path, config);
        std::fs::remove_file(path).unwrap();
    }
}

fn verify_decoded(path: &std::path::Path, config: RecordingConfig) {
    let executable = std::env::var_os("ESTELA_FFMPEG").unwrap_or_else(|| "ffmpeg".into());
    let output = Command::new(executable)
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-i"])
        .arg(path)
        .args(["-map", "0:v:0", "-vf"])
        .arg(format!("scale={}:{}", config.width, config.height))
        .args([
            "-fps_mode",
            "passthrough",
            "-pix_fmt",
            "rgba",
            "-f",
            "rawvideo",
            "pipe:1",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout.len(),
        config.frame_bytes().unwrap() * config.fps as usize
    );
    let middle =
        ((config.height as usize / 2) * config.width as usize + config.width as usize / 2) * 4;
    let pixel = &output.stdout[middle..middle + 4];
    assert!(
        pixel[0] > 220 && pixel[1] < 20 && pixel[2] < 20,
        "{pixel:?}"
    );
}

fn temporary_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "estela-{label}-{}-{unique}.partial.estela",
        std::process::id()
    ))
}
