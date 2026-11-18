use std::path::{Path, PathBuf};

use estela_model::Time;

use crate::{Error, Result};

pub mod nv12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingBackend {
    MediaFoundation,
    Nvenc,
    Vaapi { device: PathBuf },
    Software,
}

#[derive(Debug, Clone, Copy)]
pub struct RecordingConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

impl RecordingConfig {
    pub fn frame_bytes(self) -> Result<usize> {
        if self.width == 0 || self.height == 0 || ![30, 60].contains(&self.fps) {
            return Err(Error::Backend(
                "Invalid recording dimensions or frame rate".into(),
            ));
        }
        (self.width as usize)
            .checked_mul(self.height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| Error::Backend("Recording frame is too large".into()))
    }

    pub fn timestamp(self, frame: u64) -> Time {
        Time(
            (u128::from(frame) * 1_000_000_000 / u128::from(self.fps.max(1))).min(i64::MAX as u128)
                as i64,
        )
    }
}

pub struct RecordingFrame<'a> {
    pub rgba: &'a [u8],
    pub pts: Time,
}

pub trait RecordingEncoder {
    fn backend(&self) -> &RecordingBackend;
    fn write(&mut self, frame: RecordingFrame<'_>) -> Result<()>;
    fn finish(self: Box<Self>) -> Result<()>;
}

#[derive(Clone, Copy)]
pub struct RecordingEncoderFactory {
    pub probe: fn(RecordingConfig) -> Result<()>,
    pub start: fn(&Path, RecordingConfig) -> Result<Box<dyn RecordingEncoder>>,
}
