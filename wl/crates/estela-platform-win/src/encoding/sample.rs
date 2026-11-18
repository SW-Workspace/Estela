use windows::Win32::Media::MediaFoundation::{
    IMFMediaBuffer, IMFSample, MFCreateMemoryBuffer, MFCreateSample,
};

use crate::{Error, Result};

struct LockedBuffer<'a> {
    buffer: &'a IMFMediaBuffer,
}

impl Drop for LockedBuffer<'_> {
    fn drop(&mut self) {
        // SAFETY: This guard owns a successfully locked buffer.
        let _ = unsafe { self.buffer.Unlock() };
    }
}

pub(super) fn sample(bytes: &[u8], timestamp: i64, duration: i64) -> Result<IMFSample> {
    let length = u32::try_from(bytes.len()).map_err(|error| Error::Backend(error.to_string()))?;
    // SAFETY: The runtime is live and the requested allocation fits a UINT32.
    let buffer = unsafe { MFCreateMemoryBuffer(length)? };
    copy_buffer(&buffer, bytes)?;
    // SAFETY: The sample owns its buffer and uses validated recording times.
    let sample = unsafe {
        buffer.SetCurrentLength(length)?;
        let sample = MFCreateSample()?;
        sample.AddBuffer(&buffer)?;
        sample.SetSampleTime(timestamp)?;
        sample.SetSampleDuration(duration)?;
        sample
    };
    Ok(sample)
}

fn copy_buffer(buffer: &IMFMediaBuffer, bytes: &[u8]) -> Result<()> {
    let mut pointer = std::ptr::null_mut();
    let mut capacity = 0;
    // SAFETY: Both output pointers are writable and remain alive for this call.
    unsafe { buffer.Lock(&mut pointer, Some(&mut capacity), None)? };
    let lock = LockedBuffer { buffer };
    if pointer.is_null() || (capacity as usize) < bytes.len() {
        return Err(Error::Backend("MF buffer allocation is too small".into()));
    }
    // SAFETY: The locked allocation fits bytes and cannot overlap its source.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer, bytes.len()) };
    drop(lock);
    Ok(())
}
