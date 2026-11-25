use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use estela_media::{
    recording::{RecordingConfig, RecordingEncoder, RecordingFrame},
    Error, Result,
};

use super::native::NativeEncoder;

const PROBE_TIMEOUT: Duration = Duration::from_secs(10);
const PROBE_FRAMES: u64 = 3;
static PROBE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn probe(config: RecordingConfig) -> Result<()> {
    config.frame_bytes()?;
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("estela-mf-probe".into())
        .spawn(move || {
            let result = probe_on_thread(config);
            let _ = sender.send(result);
        })?;
    receiver.recv_timeout(PROBE_TIMEOUT).map_err(|error| {
        Error::Backend(format!("Media Foundation capability probe failed: {error}"))
    })?
}

fn probe_on_thread(config: RecordingConfig) -> Result<()> {
    let directory = ProbeDirectory::new()?;
    let path = directory.0.join("probe.mp4");
    let mut encoder = Box::new(NativeEncoder::start(&path, config)?);
    let rgba = vec![128; config.frame_bytes()?];
    for frame in 0..PROBE_FRAMES {
        encoder.write(RecordingFrame {
            rgba: &rgba,
            pts: config.timestamp(frame),
        })?;
    }
    encoder.finish()?;
    validate_fragmented_media(&path)
}

pub fn validate_fragmented_media(path: &Path) -> Result<()> {
    let bytes = std::fs::read(path)?;
    let mut offset = 0;
    let mut file_type = false;
    let mut movie = false;
    let mut fragment = false;
    let mut media = false;
    while bytes.len().saturating_sub(offset) >= 8 {
        let short = u32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| Error::Backend("Invalid MP4 box".into()))?,
        );
        let (size, header) = box_size(&bytes[offset..], short)?;
        if size < header || size > bytes.len() - offset {
            return Err(Error::Backend("Invalid fragmented MP4 box bounds".into()));
        }
        match &bytes[offset + 4..offset + 8] {
            b"ftyp" => file_type = offset == 0,
            b"moov" => movie = true,
            b"moof" => fragment = true,
            b"mdat" => media |= size > header,
            _ => {}
        }
        offset += size;
    }
    if offset != bytes.len() || !file_type || !movie || !fragment || !media {
        return Err(Error::Backend(
            "Media Foundation did not produce fragmented video".into(),
        ));
    }
    Ok(())
}

fn box_size(bytes: &[u8], short: u32) -> Result<(usize, usize)> {
    match short {
        0 => Ok((bytes.len(), 8)),
        1 => {
            let raw = bytes
                .get(8..16)
                .ok_or_else(|| Error::Backend("Truncated extended MP4 box".into()))?;
            let size = u64::from_be_bytes(
                raw.try_into()
                    .map_err(|_| Error::Backend("Invalid extended MP4 box".into()))?,
            );
            let size = usize::try_from(size).map_err(|error| Error::Backend(error.to_string()))?;
            Ok((size, 16))
        }
        size => Ok((size as usize, 8)),
    }
}

struct ProbeDirectory(PathBuf);

impl ProbeDirectory {
    fn new() -> Result<Self> {
        let sequence = PROBE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| Error::Backend(error.to_string()))?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "estela-mf-probe-{}-{unique}-{sequence}",
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
