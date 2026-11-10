use estela_media::caps::Capabilities;

use crate::{device::GpuSelector, Error, Result};

pub fn probe(_gpu: &GpuSelector) -> Result<Capabilities> {
    Err(Error::NotImplemented)
}
