use std::path::Path;

use estela_media::recording::{RecordingBackend, RecordingConfig, RecordingEncoder};
use estela_media_ffmpeg::encode::{self, Encoder};

use crate::{Error, Result};

pub(super) struct PreparedEncoder {
    config: RecordingConfig,
    backend: RecordingBackend,
}

impl PreparedEncoder {
    pub(super) fn start(self, path: &Path) -> Result<Box<dyn RecordingEncoder>> {
        if path.exists() {
            return Err(Error::Recording(
                "Recording destination already exists".into(),
            ));
        }
        Ok(Box::new(Encoder::start_backend(
            path,
            self.config,
            self.backend,
        )?))
    }
}

pub(super) fn prepare(config: RecordingConfig) -> Result<PreparedEncoder> {
    prepare_with_backends(config, crate::platform::recording_backends())
}

fn prepare_with_backends(
    config: RecordingConfig,
    backends: Vec<RecordingBackend>,
) -> Result<PreparedEncoder> {
    let mut failures = Vec::new();
    for backend in backends {
        match encode::probe(config, &backend) {
            Ok(()) => {
                eprintln!("Recording encoder selected: {backend:?}");
                return Ok(PreparedEncoder { config, backend });
            }
            Err(error) => {
                eprintln!("Recording encoder {backend:?} is unavailable: {error}");
                failures.push(format!("{backend:?}: {error}"));
            }
        }
    }
    Err(Error::Recording(format!(
        "No recording encoder is available: {}",
        failures.join("; "),
    )))
}

#[cfg(test)]
mod tests;
