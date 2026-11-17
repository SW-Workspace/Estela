use std::path::Path;

use estela_media::recording::{RecordingBackend, RecordingConfig, RecordingEncoder};
use estela_media_ffmpeg::encode::{self, Encoder};

use crate::{Error, Result};

pub(super) struct PreparedEncoder {
    config: RecordingConfig,
    backends: Vec<RecordingBackend>,
}

impl PreparedEncoder {
    pub(super) fn start(self, path: &Path) -> Result<Box<dyn RecordingEncoder>> {
        let mut failures = Vec::new();
        for (index, backend) in self.backends.iter().enumerate() {
            if path.exists() {
                return Err(Error::Recording(
                    "Recording destination already exists".into(),
                ));
            }
            let opened = if index == 0 {
                open(path, self.config, backend)
            } else {
                probe(self.config, backend).and_then(|()| open(path, self.config, backend))
            };
            match opened {
                Ok(encoder) => {
                    eprintln!("Recording encoder started: {:?}", encoder.backend());
                    return Ok(encoder);
                }
                Err(error) => {
                    eprintln!("Recording encoder {backend:?} could not start: {error}");
                    failures.push(format!("{backend:?}: {error}"));
                }
            }
        }
        Err(unavailable(failures))
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
    for (index, backend) in backends.iter().enumerate() {
        match probe(config, backend) {
            Ok(()) => {
                eprintln!("Recording encoder selected: {backend:?}");
                return Ok(PreparedEncoder {
                    config,
                    backends: backends[index..].to_vec(),
                });
            }
            Err(error) => {
                eprintln!("Recording encoder {backend:?} is unavailable: {error}");
                failures.push(format!("{backend:?}: {error}"));
            }
        }
    }
    Err(unavailable(failures))
}

fn probe(config: RecordingConfig, backend: &RecordingBackend) -> Result<()> {
    if let Some(factory) = crate::platform::native_factory(backend) {
        return (factory.probe)(config).map_err(|error| Error::Recording(error.to_string()));
    }
    Ok(encode::probe(config, backend)?)
}

fn open(
    path: &Path,
    config: RecordingConfig,
    backend: &RecordingBackend,
) -> Result<Box<dyn RecordingEncoder>> {
    if let Some(factory) = crate::platform::native_factory(backend) {
        return (factory.start)(path, config).map_err(|error| Error::Recording(error.to_string()));
    }
    Ok(Box::new(Encoder::start_backend(
        path,
        config,
        backend.clone(),
    )?))
}

fn unavailable(failures: Vec<String>) -> Error {
    Error::Recording(format!(
        "No recording encoder is available: {}",
        failures.join("; ")
    ))
}

#[cfg(test)]
mod tests;
