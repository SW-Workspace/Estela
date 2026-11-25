use std::path::Path;

use estela_media::recording::{
    nv12::Nv12Buffer, RecordingBackend, RecordingConfig, RecordingEncoder, RecordingFrame,
};

use super::{sample, writer::Writer};

const NANOSECONDS_PER_MF_TICK: i64 = 100;

pub(super) struct NativeEncoder {
    writer: Writer,
    pixels: Nv12Buffer,
    config: RecordingConfig,
    backend: RecordingBackend,
    frames: u64,
}

impl NativeEncoder {
    pub(super) fn start(path: &Path, config: RecordingConfig) -> estela_media::Result<Self> {
        let pixels = Nv12Buffer::new(config)?;
        let writer = Writer::start(path, config, pixels.size()).map_err(error)?;
        Ok(Self {
            writer,
            pixels,
            config,
            backend: RecordingBackend::MediaFoundation,
            frames: 0,
        })
    }
}

impl RecordingEncoder for NativeEncoder {
    fn backend(&self) -> &RecordingBackend {
        &self.backend
    }

    fn write(&mut self, frame: RecordingFrame<'_>) -> estela_media::Result<()> {
        let next = self
            .frames
            .checked_add(1)
            .ok_or_else(|| estela_media::Error::Backend("Recording frame count overflow".into()))?;
        if frame.pts != self.config.timestamp(self.frames) {
            return Err(estela_media::Error::Backend(
                "Unexpected recording frame timestamp".into(),
            ));
        }
        let timestamp = frame.pts.0 / NANOSECONDS_PER_MF_TICK;
        let duration = self.config.timestamp(next).0 / NANOSECONDS_PER_MF_TICK - timestamp;
        let bytes = self.pixels.convert(frame.rgba)?;
        let sample = sample::sample(bytes, timestamp, duration).map_err(error)?;
        self.writer.write(&sample).map_err(error)?;
        self.frames = next;
        Ok(())
    }

    fn finish(mut self: Box<Self>) -> estela_media::Result<()> {
        self.writer.finish().map_err(error)
    }
}

fn error(error: crate::Error) -> estela_media::Error {
    estela_media::Error::Backend(error.to_string())
}
