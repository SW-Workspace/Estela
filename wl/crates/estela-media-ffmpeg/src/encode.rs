use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
};

use estela_media::recording::{
    RecordingBackend, RecordingConfig, RecordingEncoder, RecordingFrame,
};

use crate::{Error, Result};

mod probe;
mod settings;

pub use probe::probe;

pub struct Encoder {
    child: Child,
    input: Option<ChildStdin>,
    log: PathBuf,
    config: RecordingConfig,
    backend: RecordingBackend,
    frames: u64,
}

pub fn executable() -> std::ffi::OsString {
    crate::runtime::executable()
}

pub fn available() -> Result<()> {
    let mut command = Command::new(executable());
    let output = command
        .args(["-version"])
        .output()
        .map_err(|error| crate::runtime::launch_error(&command, error))?;
    if !output.status.success() {
        return Err(Error::Backend("FFmpeg is unavailable".into()));
    }
    Ok(())
}

impl Encoder {
    pub fn start(path: &Path, size: (u32, u32), fps: u32) -> Result<Self> {
        Self::start_backend(
            path,
            RecordingConfig {
                width: size.0,
                height: size.1,
                fps,
            },
            RecordingBackend::Software,
        )
    }

    pub fn start_backend(
        path: &Path,
        config: RecordingConfig,
        backend: RecordingBackend,
    ) -> Result<Self> {
        let log = path.with_extension("log");
        let mut command =
            settings::command(path, config, &backend, std::ffi::OsStr::new("pipe:0"))?;
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(File::create(&log)?);
        let mut child = crate::runtime::spawn(&mut command)?;
        let input = child.stdin.take();
        Ok(Self {
            child,
            input,
            log,
            config,
            backend,
            frames: 0,
        })
    }

    pub fn write(&mut self, rgba: &[u8]) -> Result<()> {
        let expected = self
            .config
            .frame_bytes()
            .map_err(|error| Error::Backend(error.to_string()))?;
        if rgba.len() != expected {
            return Err(Error::Backend(
                "Recording frame size does not match encoder configuration".into(),
            ));
        }
        self.input
            .as_mut()
            .ok_or_else(|| Error::Backend("Encoder is closed".into()))?
            .write_all(rgba)?;
        self.frames += 1;
        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        self.input.take();
        if !self.child.wait()?.success() {
            let message = std::fs::read_to_string(&self.log).unwrap_or_default();
            return Err(Error::Backend(format!(
                "FFmpeg could not finish recording: {message}"
            )));
        }
        let _ = std::fs::remove_file(&self.log);
        Ok(())
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        self.input.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl RecordingEncoder for Encoder {
    fn backend(&self) -> &RecordingBackend {
        &self.backend
    }

    fn write(&mut self, frame: RecordingFrame<'_>) -> estela_media::Result<()> {
        if frame.pts != self.config.timestamp(self.frames) {
            return Err(estela_media::Error::Backend(
                "Unexpected recording frame timestamp".into(),
            ));
        }
        Encoder::write(self, frame.rgba)
            .map_err(|error| estela_media::Error::Backend(error.to_string()))
    }

    fn finish(self: Box<Self>) -> estela_media::Result<()> {
        Encoder::finish(*self).map_err(|error| estela_media::Error::Backend(error.to_string()))
    }
}
