use std::{
    io::Read,
    path::Path,
    process::{Child, ChildStdout, Command, Stdio},
};

use crate::{Error, Result};

pub struct Decoder {
    child: Child,
    output: ChildStdout,
    pub size: (u32, u32),
}

impl Decoder {
    pub fn start(path: &Path, size: (u32, u32), seconds: f64) -> Result<Self> {
        let mut child = Command::new(crate::encode::executable())
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-nostdin",
                "-ss",
                &format!("{seconds:.6}"),
                "-i",
            ])
            .arg(path)
            .args([
                "-an",
                "-vf",
                &format!("scale={}:{}", size.0, size.1),
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgba",
                "pipe:1",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let output = child
            .stdout
            .take()
            .ok_or_else(|| Error::Backend("Missing decoder output".into()))?;
        Ok(Self {
            child,
            output,
            size,
        })
    }

    pub fn next_frame(&mut self) -> Result<Option<Vec<u8>>> {
        let mut frame = vec![0; self.size.0 as usize * self.size.1 as usize * 4];
        match self.output.read_exact(&mut frame) {
            Ok(()) => Ok(Some(frame)),
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                if self.child.wait()?.success() {
                    Ok(None)
                } else {
                    Err(Error::Backend("Video decoding failed".into()))
                }
            }
            Err(error) => Err(error.into()),
        }
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
