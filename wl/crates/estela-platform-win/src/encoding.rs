use std::path::Path;

use estela_media::recording::{
    RecordingBackend, RecordingConfig, RecordingEncoder, RecordingEncoderFactory,
};

mod native;
mod probe;
mod runtime;
mod sample;
mod types;
mod writer;

pub use probe::validate_fragmented_media;

pub fn recording_backends() -> Vec<RecordingBackend> {
    vec![
        RecordingBackend::MediaFoundation,
        RecordingBackend::Software,
    ]
}

pub fn native_factory(backend: &RecordingBackend) -> Option<RecordingEncoderFactory> {
    (*backend == RecordingBackend::MediaFoundation).then_some(RecordingEncoderFactory {
        probe: probe::probe,
        start: start_native,
    })
}

pub fn start_native(
    path: &Path,
    config: RecordingConfig,
) -> estela_media::Result<Box<dyn RecordingEncoder>> {
    Ok(Box::new(native::NativeEncoder::start(path, config)?))
}
