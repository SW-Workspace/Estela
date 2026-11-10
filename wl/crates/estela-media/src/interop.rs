use estela_model::{ColorInfo, Time};

use crate::{
    frame::{GpuFrame, NativeFrame},
    Result,
};

pub trait GpuInterop: Send + Sync {
    fn import(&self, frame: NativeFrame, pts: Time, color: ColorInfo) -> Result<GpuFrame>;
}
