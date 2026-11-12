use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
};

use crate::{Error, Result};

pub struct Encoder {
    child: Child,
    input: Option<ChildStdin>,
    log: PathBuf,
}

pub fn executable() -> std::ffi::OsString {
    std::env::var_os("ESTELA_FFMPEG").unwrap_or_else(|| "ffmpeg".into())
}

pub fn available() -> Result<()> {
    let output = Command::new(executable()).args(["-version"]).output()?;
    if !output.status.success() {
        return Err(Error::Backend("FFmpeg is unavailable".into()));
    }
    Ok(())
}

impl Encoder {
    pub fn start(path: &Path, size: (u32, u32), fps: u32) -> Result<Self> {
        let log = path.with_extension("log");
        let mut child = Command::new(executable())
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-nostdin",
                "-n",
                "-f",
                "rawvideo",
                "-pixel_format",
                "rgba",
                "-video_size",
            ])
            .arg(format!("{}x{}", size.0, size.1))
            .args([
                "-framerate",
                &fps.to_string(),
                "-i",
                "pipe:0",
                "-an",
                "-c:v",
                "mpeg4",
                "-q:v",
                "3",
                "-pix_fmt",
                "yuv420p",
                "-vf",
                "pad=ceil(iw/2)*2:ceil(ih/2)*2",
                "-g",
            ])
            .arg(fps.to_string())
            .args([
                "-movflags",
                "+frag_keyframe+empty_moov+default_base_moof",
                "-f",
                "mp4",
            ])
            .arg(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::from(File::create(&log)?))
            .spawn()?;
        let input = child.stdin.take();
        Ok(Self { child, input, log })
    }

    pub fn write(&mut self, rgba: &[u8]) -> Result<()> {
        self.input
            .as_mut()
            .ok_or_else(|| Error::Backend("Encoder is closed".into()))?
            .write_all(rgba)?;
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
