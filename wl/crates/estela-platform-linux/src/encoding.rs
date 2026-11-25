use std::path::Path;

use estela_media::recording::RecordingBackend;

pub fn recording_backends() -> Vec<RecordingBackend> {
    let mut backends = Vec::new();
    if Path::new("/dev/nvidia0").exists() {
        backends.push(RecordingBackend::Nvenc);
    }
    if let Ok(entries) = std::fs::read_dir("/dev/dri") {
        let mut devices: Vec<_> = entries
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("renderD"))
            .map(|entry| entry.path())
            .collect();
        devices.sort();
        for device in devices {
            backends.push(RecordingBackend::Vaapi { device });
        }
    }
    backends.push(RecordingBackend::Software);
    backends
}
