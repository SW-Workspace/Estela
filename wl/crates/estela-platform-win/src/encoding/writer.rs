use std::path::{Path, PathBuf};

use windows::{
    core::{HSTRING, PCWSTR},
    Win32::Media::MediaFoundation::{
        IMFByteStream, IMFMediaSink, IMFSample, IMFSinkWriter, MFCreateFMPEG4MediaSink,
        MFCreateFile, MFCreateSinkWriterFromMediaSink, MF_ACCESSMODE_WRITE, MF_E_SHUTDOWN,
        MF_FILEFLAGS_NONE, MF_OPENMODE_FAIL_IF_EXIST,
    },
};

use estela_media::recording::RecordingConfig;

use crate::{Error, Result};

use super::{runtime::Runtime, types};

const VIDEO_STREAM_INDEX: u32 = 0;

pub(super) struct Writer {
    writer: Option<IMFSinkWriter>,
    sink: Option<IMFMediaSink>,
    stream: Option<IMFByteStream>,
    path: PathBuf,
    discard: bool,
    _runtime: Runtime,
}

impl Writer {
    pub(super) fn start(path: &Path, config: RecordingConfig, size: (u32, u32)) -> Result<Self> {
        let mut writer = Self {
            writer: None,
            sink: None,
            stream: None,
            path: std::path::absolute(path)?,
            discard: false,
            _runtime: Runtime::start()?,
        };
        writer.initialize(config, size)?;
        writer.discard = false;
        Ok(writer)
    }

    fn initialize(&mut self, config: RecordingConfig, size: (u32, u32)) -> Result<()> {
        let output = types::output_type(config, size.0, size.1)?;
        let input = types::input_type(config, size.0, size.1)?;
        let attributes = types::attributes()?;
        let parameters = types::encoding_parameters(config)?;
        let name = HSTRING::from(self.path.as_os_str());
        if name.contains(&0) {
            return Err(Error::Backend(
                "Recording path contains a null character".into(),
            ));
        }
        // SAFETY: The UTF-16 path is live and existing files fail.
        let stream = unsafe {
            MFCreateFile(
                MF_ACCESSMODE_WRITE,
                MF_OPENMODE_FAIL_IF_EXIST,
                MF_FILEFLAGS_NONE,
                PCWSTR(name.as_ptr()),
            )?
        };
        self.discard = true;
        self.stream = Some(stream.clone());
        // SAFETY: The live byte stream and H.264 type outlive this call.
        let sink = unsafe { MFCreateFMPEG4MediaSink(&stream, &output, None)? };
        self.sink = Some(sink.clone());
        // SAFETY: The writer retains this owned sink and its attribute store.
        let writer = unsafe { MFCreateSinkWriterFromMediaSink(&sink, &attributes)? };
        self.writer = Some(writer.clone());
        // SAFETY: The single video stream has zero-based writer index zero.
        unsafe {
            writer.SetInputMediaType(VIDEO_STREAM_INDEX, &input, &parameters)?;
            writer.BeginWriting()?;
        }
        Ok(())
    }

    pub(super) fn write(&self, sample: &IMFSample) -> Result<()> {
        let writer = self.writer.as_ref().ok_or_else(closed)?;
        // SAFETY: The live writer accepts an owned, timed NV12 video sample.
        unsafe { writer.WriteSample(VIDEO_STREAM_INDEX, sample)? };
        Ok(())
    }

    pub(super) fn finish(&mut self) -> Result<()> {
        let writer = self.writer.as_ref().ok_or_else(closed)?;
        // SAFETY: The synchronous writer has received its last sample.
        unsafe { writer.Finalize()? };
        self.close()
    }

    fn close(&mut self) -> Result<()> {
        drop(self.writer.take());
        let flushed = self
            .stream
            .as_ref()
            .map(|stream| {
                // SAFETY: The live byte stream is flushed before sink shutdown.
                unsafe { stream.Flush() }
            })
            .unwrap_or(Ok(()));
        let shutdown = self
            .sink
            .take()
            .map(|sink| {
                // SAFETY: Writer references are released before sink shutdown.
                unsafe { sink.Shutdown() }
            })
            .unwrap_or(Ok(()));
        let closed = self
            .stream
            .take()
            .map(|stream| {
                // SAFETY: Close is paired with this owned stream's creation.
                unsafe { stream.Close() }
            })
            .unwrap_or(Ok(()));
        allow_shutdown(flushed)?;
        allow_shutdown(shutdown)?;
        allow_shutdown(closed)
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        let _ = self.close();
        if self.discard {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

fn closed() -> Error {
    Error::Backend("Media Foundation writer is closed".into())
}

fn allow_shutdown(result: windows::core::Result<()>) -> Result<()> {
    match result {
        Err(error) if error.code() == MF_E_SHUTDOWN => Ok(()),
        result => Ok(result?),
    }
}
