use estela_model::Time;

use crate::{frame::GpuFrame, Result};

pub struct EncodedPacket {
    pub pts: Time,
    pub dts: Time,
    pub keyframe: crate::frame::FrameKind,
    pub data: Vec<u8>,
}
pub trait Encoder: Send {
    fn encode(&mut self, frame: &GpuFrame) -> Result<Vec<EncodedPacket>>;
    fn finish(&mut self) -> Result<Vec<EncodedPacket>>;
}
