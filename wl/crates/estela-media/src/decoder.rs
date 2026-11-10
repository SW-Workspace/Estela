use estela_model::Time;

use crate::{frame::GpuFrame, Result};

pub struct CompressedSample {
    pub pts: Time,
    pub dts: Time,
    pub keyframe: crate::frame::FrameKind,
    pub data: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeBackend {
    Hardware(String),
    Software,
}
pub trait Decoder: Send {
    fn send(&mut self, sample: &CompressedSample) -> Result<()>;
    fn receive(&mut self) -> Result<Option<GpuFrame>>;
    fn flush(&mut self);
    fn backend(&self) -> DecodeBackend;
}
