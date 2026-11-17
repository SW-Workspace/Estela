use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use estela_media::recording::{RecordingBackend, RecordingConfig};

use crate::{Error, Result};

const PROBE_TIMEOUT: Duration = Duration::from_secs(10);
const PROBE_INTERVAL: Duration = Duration::from_millis(20);
const PROBE_FRAMES: usize = 3;
static PROBE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub fn probe(config: RecordingConfig, backend: &RecordingBackend) -> Result<()> {
    let frame_bytes = config
        .frame_bytes()
        .map_err(|error| Error::Backend(error.to_string()))?;
    let directory = ProbeDirectory::new()?;
    let input = directory.0.join("input.rgba");
    let output = directory.0.join("output.mp4");
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&input)?;
    let frame = vec![128; frame_bytes];
    for _ in 0..PROBE_FRAMES {
        file.write_all(&frame)?;
    }
    drop(file);
    let mut command = super::settings::command(&output, config, backend, input.as_os_str())?;
    run(&mut command, &directory)?;
    let mut decode = Command::new(super::executable());
    let decoded = directory.0.join("decoded.rgba");
    decode
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-i"])
        .arg(output)
        .args(["-map", "0:v:0", "-frames:v", "1", "-vf"])
        .arg(format!("scale={}:{}", config.width, config.height))
        .args(["-pix_fmt", "rgba", "-f", "rawvideo"])
        .arg(&decoded);
    run(&mut decode, &directory)?;
    if std::fs::metadata(decoded)?.len() != frame_bytes as u64 {
        return Err(Error::Backend(
            "Encoder probe did not produce a decodable frame".into(),
        ));
    }
    Ok(())
}

fn run(command: &mut Command, directory: &ProbeDirectory) -> Result<()> {
    let log = directory.0.join("probe.log");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(File::create(&log)?);
    let mut child = crate::runtime::spawn(command)?;
    let started = Instant::now();
    loop {
        let status = match child.try_wait() {
            Ok(status) => status,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.into());
            }
        };
        if let Some(status) = status {
            return if status.success() {
                Ok(())
            } else {
                Err(Error::Backend(std::fs::read_to_string(&log)?))
            };
        }
        if started.elapsed() >= PROBE_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Error::Backend("Encoder capability probe timed out".into()));
        }
        std::thread::sleep(PROBE_INTERVAL);
    }
}

struct ProbeDirectory(PathBuf);

impl ProbeDirectory {
    fn new() -> Result<Self> {
        let sequence = PROBE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "estela-encoder-probe-{}-{sequence}",
            std::process::id(),
        ));
        std::fs::create_dir(&path)?;
        Ok(Self(path))
    }
}

impl Drop for ProbeDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
